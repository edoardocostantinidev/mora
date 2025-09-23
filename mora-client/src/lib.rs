use mora_core::{
    entities::cluster_status::ClusterStatus,
    result::{MoraError, MoraResult},
};

use reqwest::{StatusCode, Url};
use url::ParseError;

#[derive(Debug, Clone)]
pub struct MoraClient {
    base_url: String,
    port: u16,
    http_client: reqwest::Client,
}

impl MoraClient {
    pub fn new(base_url: String, port: u16) -> Self {
        Self {
            base_url,
            port,
            http_client: reqwest::Client::new(),
        }
    }

    fn build_url(&self, path: &str) -> MoraResult<Url> {
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

    pub async fn get_cluster_status(&self) -> MoraResult<ClusterStatus> {
        let url = self.build_url("health")?;
        let response = self
            .http_client
            .clone()
            .get(url)
            .send()
            .await
            .map_err(handle_request_error)?;

        match response.status() {
            StatusCode::OK => {
                let raw_body = response.text().await.map_err(handle_request_error)?;
                let status = serde_json::from_str::<ClusterStatus>(&raw_body)
                    .map_err(|err| handle_decode_error(err, &raw_body))?;
                Ok(status)
            }
            _ => Ok(ClusterStatus::Offline),
        }
    }
}

fn handle_request_error(error: reqwest::Error) -> MoraError {
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
