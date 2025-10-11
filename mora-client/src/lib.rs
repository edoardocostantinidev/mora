use mora_core::{
    models::{
        channels::ListChannelsResponse,
        connections::ConnectionsInfo,
        health::{ClusterStatus, ClusterStatusData},
        queues::ListQueuesResponse,
    },
    result::{MoraError, MoraResult},
};

use mora_proto::health::{
    self, health_check_response::Status, ClusterStatusData as ProtoClusterStatusData,
    HealthCheckRequest,
};
use reqwest::Url;
use url::ParseError;

const MORA_ID_KEY_HEADER: &str = "MORA-ID-KEY";

#[derive(Debug, Clone)]
pub struct MoraClient {
    base_url: String,
    port: u16,
    http_client: reqwest::Client,
    health_client: health::health_service_client::HealthServiceClient<tonic::transport::Channel>,
    id_key: String,
}

impl MoraClient {
    pub async fn new(base_url: String, port: u16, id_key: String) -> MoraResult<Self> {
        let channel = tonic::transport::Channel::from_shared(format!("http://{base_url}:{port}"))
            .map_err(|e| MoraError::GenericError(format!("Invalid base URL: {e}")))?
            .connect()
            .await
            .map_err(|e| MoraError::ConnectionError(format!("Failed to connect: {e}")))?;
        let health_client = health::health_service_client::HealthServiceClient::new(channel);

        Ok(Self {
            base_url,
            port,
            http_client: reqwest::Client::new(),
            health_client,
            id_key,
        })
    }

    pub fn build_url(&self, path: &str) -> MoraResult<Url> {
        Url::parse(
            format!(
                "http://{base_url}:{port}/{path}",
                base_url = self.base_url,
                port = self.port,
                path = path
            )
            .as_str(),
        )
        .map_err(handle_url_error)
    }

    async fn get_request<T: serde::de::DeserializeOwned>(&self, path: &str) -> MoraResult<T> {
        let url = self.build_url(path)?;
        let response = self
            .http_client
            .clone()
            .get(url)
            .header(MORA_ID_KEY_HEADER, self.id_key.to_owned())
            .send()
            .await;

        match response {
            Ok(response) => {
                let raw_body = response.text().await.map_err(handle_request_error)?;
                Ok(serde_json::from_str::<T>(&raw_body)
                    .map_err(|e| handle_decode_error(e, &raw_body))?)
            }
            Err(error) => Err(handle_request_error(error)),
        }
    }

    pub async fn get_cluster_status(&self) -> MoraResult<ClusterStatus> {
        let cluster_status = self
            .clone()
            .health_client
            .get_cluster_status(HealthCheckRequest {})
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner()
            .status;

        match cluster_status {
            Some(Status::Online(ProtoClusterStatusData {
                current_time_in_ns: bytes,
                version,
            })) => {
                let current_time_in_ns_bytes: [u8; 16] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| MoraError::GenericError("Invalid ns time format".to_string()))?;
                let current_time_in_ns = u128::from_le_bytes(current_time_in_ns_bytes);
                Ok(ClusterStatus::Online(ClusterStatusData {
                    current_time_in_ns,
                    version,
                }))
            }
            _ => Ok(ClusterStatus::Offline),
        }
    }

    pub async fn get_connections_info(&self) -> MoraResult<ConnectionsInfo> {
        self.get_request::<ConnectionsInfo>("connections/info")
            .await
    }

    pub async fn get_queues(&self) -> MoraResult<ListQueuesResponse> {
        self.get_request::<ListQueuesResponse>("queues").await
    }

    pub async fn get_channels(&self) -> MoraResult<ListChannelsResponse> {
        self.get_request::<ListChannelsResponse>("channels").await
    }
}

fn handle_request_error(error: reqwest::Error) -> MoraError {
    if error.is_connect() {
        return MoraError::ConnectionError(format!("failed to connect to server: {error}"));
    }

    MoraError::GenericError(format!("error making request: {error}"))
}

fn handle_decode_error(error: serde_json::Error, response: &str) -> MoraError {
    MoraError::GenericError(format!(
        "error decoding response: {error}, response: {:?}",
        response
    ))
}

fn handle_url_error(error: ParseError) -> MoraError {
    MoraError::GenericError(format!("error parsing url: {error}"))
}
