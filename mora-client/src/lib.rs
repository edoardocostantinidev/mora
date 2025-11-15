use mora_core::{
    models::{
        channels::ListChannelsResponse,
        connections::ConnectionsInfo,
        health::{ClusterStatus, ClusterStatusData},
        queues::ListQueuesResponse,
    },
    result::{MoraError, MoraResult},
};

use mora_proto::{
    channels::{channel_service_client::ChannelServiceClient, ListChannelsRequest},
    connections::{
        connection_service_client::ConnectionServiceClient, GetConnectionsInfoRequest,
    },
    health::{
        health_check_response::Status, health_service_client::HealthServiceClient,
        ClusterStatusData as ProtoClusterStatusData, HealthCheckRequest,
    },
    queues::{queue_service_client::QueueServiceClient, ListQueuesRequest},
};

#[derive(Debug, Clone)]
pub struct MoraClient {
    health_client: HealthServiceClient<tonic::transport::Channel>,
    queue_client: QueueServiceClient<tonic::transport::Channel>,
    channel_client: ChannelServiceClient<tonic::transport::Channel>,
    connection_client: ConnectionServiceClient<tonic::transport::Channel>,
}

impl MoraClient {
    pub async fn new(base_url: String, port: u16, _id_key: String) -> MoraResult<Self> {
        let channel = tonic::transport::Channel::from_shared(format!("http://{base_url}:{port}"))
            .map_err(|e| MoraError::GenericError(format!("Invalid base URL: {e}")))?
            .connect()
            .await
            .map_err(|e| MoraError::ConnectionError(format!("Failed to connect: {e}")))?;

        let health_client = HealthServiceClient::new(channel.clone());
        let queue_client = QueueServiceClient::new(channel.clone());
        let channel_client = ChannelServiceClient::new(channel.clone());
        let connection_client = ConnectionServiceClient::new(channel);

        Ok(Self {
            health_client,
            queue_client,
            channel_client,
            connection_client,
        })
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
        let response = self
            .clone()
            .connection_client
            .get_connections_info(GetConnectionsInfoRequest {})
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        Ok(ConnectionsInfo {
            clients_connected: response.clients_connected as usize,
        })
    }

    pub async fn get_queues(&self) -> MoraResult<ListQueuesResponse> {
        let response = self
            .clone()
            .queue_client
            .list_queues(ListQueuesRequest {})
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        let queues = response
            .queues
            .into_iter()
            .map(|q| mora_core::models::queues::Queue {
                id: q.id,
                pending_events_count: q.pending_events_count as u128,
            })
            .collect();

        Ok(ListQueuesResponse { queues })
    }

    pub async fn get_channels(&self) -> MoraResult<ListChannelsResponse> {
        let response = self
            .clone()
            .channel_client
            .list_channels(ListChannelsRequest {})
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        let channels = response
            .channels
            .into_iter()
            .map(|c| mora_core::models::channels::Channel {
                channel_id: c.channel_id,
                queues: c.queues,
                buffer_options: mora_core::models::channels::BufferOptions {
                    size: c.buffer_options.as_ref().map(|b| b.size as usize).unwrap_or(0),
                    time: c.buffer_options.as_ref().map(|b| b.time as u128).unwrap_or(0),
                },
                msec_from_last_op: c.msec_from_last_op as usize,
            })
            .collect();

        Ok(ListChannelsResponse { channels })
    }
}
