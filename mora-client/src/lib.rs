use mora_core::{
    models::{
        channels::ListChannelsResponse,
        events::Event,
        health::{ClusterStatus, ClusterStatusData},
        queues::ListQueuesResponse,
    },
    result::{MoraError, MoraResult},
};

use mora_proto::{
    channels::{
        channel_service_client::ChannelServiceClient, BufferOptions, CreateChannelRequest,
        DeleteChannelRequest, GetChannelEventsRequest, ListChannelsRequest,
    },
    health::{
        health_check_response::Status, health_service_client::HealthServiceClient,
        ClusterStatusData as ProtoClusterStatusData, HealthCheckRequest,
    },
    queues::{queue_service_client::QueueServiceClient, DeleteQueueRequest, ListQueuesRequest},
};

#[derive(Debug, Clone)]
pub struct MoraClient {
    health_client: Option<HealthServiceClient<tonic::transport::Channel>>,
    queue_client: Option<QueueServiceClient<tonic::transport::Channel>>,
    channel_client: Option<ChannelServiceClient<tonic::transport::Channel>>,
    channel: Option<tonic::transport::Channel>,
    endpoint: Option<tonic::transport::Endpoint>,
    connected: bool,
}

impl MoraClient {
    pub async fn new(base_url: String, port: u16, _id_key: String) -> MoraResult<Self> {
        let endpoint = tonic::transport::Channel::from_shared(format!("http://{base_url}:{port}"))
            .map_err(|e| MoraError::GenericError(format!("Invalid base URL: {e}")))?;

        Ok(Self {
            health_client: None,
            queue_client: None,
            channel_client: None,
            channel: None,
            endpoint: Some(endpoint),
            connected: false,
        })
    }

    pub async fn get_cluster_status(&mut self) -> MoraResult<ClusterStatus> {
        if !self.connected {
            self.connect().await?;
        }

        let cluster_status = self
            .clone()
            .health_client
            .ok_or(MoraError::NotConnected)?
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

    pub async fn get_queues(&mut self) -> MoraResult<ListQueuesResponse> {
        if !self.connected {
            self.connect().await?;
        }

        let response = self
            .clone()
            .queue_client
            .ok_or(MoraError::NotConnected)?
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

    pub async fn delete_queue(&mut self, queue_id: String) -> MoraResult<String> {
        if !self.connected {
            self.connect().await?;
        }

        let response = self
            .clone()
            .queue_client
            .ok_or(MoraError::NotConnected)?
            .delete_queue(DeleteQueueRequest { queue_id })
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        Ok(response.message)
    }

    pub async fn get_channels(&mut self) -> MoraResult<ListChannelsResponse> {
        if !self.connected {
            self.connect().await?;
        }

        let response = self
            .clone()
            .channel_client
            .ok_or(MoraError::NotConnected)?
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
                    size: c
                        .buffer_options
                        .as_ref()
                        .map(|b| b.size as usize)
                        .unwrap_or(0),
                    time: c
                        .buffer_options
                        .as_ref()
                        .map(|b| b.time as u128)
                        .unwrap_or(0),
                },
                msec_from_last_op: c.msec_from_last_op as usize,
            })
            .collect();

        Ok(ListChannelsResponse { channels })
    }

    pub async fn create_channel(
        &mut self,
        queues: Vec<String>,
        buffer_size: u64,
        buffer_time: u64,
    ) -> MoraResult<String> {
        if !self.connected {
            self.connect().await?;
        }

        let response = self
            .clone()
            .channel_client
            .ok_or(MoraError::NotConnected)?
            .create_channel(CreateChannelRequest {
                queues,
                buffer_options: Some(BufferOptions {
                    size: buffer_size,
                    time: buffer_time,
                }),
            })
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        Ok(response.channel_id)
    }

    pub async fn delete_channel(&mut self, channel_id: String) -> MoraResult<()> {
        if !self.connected {
            self.connect().await?;
        }

        self.clone()
            .channel_client
            .ok_or(MoraError::NotConnected)?
            .delete_channel(DeleteChannelRequest { channel_id })
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_channel_events(
        &mut self,
        channel_id: String,
        delete: bool,
    ) -> MoraResult<Vec<Event>> {
        if !self.connected {
            self.connect().await?;
        }

        let response = self
            .clone()
            .channel_client
            .ok_or(MoraError::NotConnected)?
            .get_channel_events(GetChannelEventsRequest { channel_id, delete })
            .await
            .map_err(|e| MoraError::GenericError(e.to_string()))?
            .into_inner();

        let events = response
            .events
            .into_iter()
            .map(|e| Event {
                timestamp: e.timestamp.parse::<u128>().unwrap_or(0),
                queue_name: e.queue_name,
                data: e.data,
            })
            .collect();

        Ok(events)
    }

    async fn connect(&mut self) -> MoraResult<()> {
        let channel = self
            .endpoint
            .as_ref()
            .ok_or(MoraError::GenericError("Invalid endpoint".to_string()))?
            .connect()
            .await
            .map_err(|e| MoraError::ConnectionError(format!("grpc connection failed: {e}")))?;

        self.health_client = Some(HealthServiceClient::new(channel.clone()));
        self.queue_client = Some(QueueServiceClient::new(channel.clone()));
        self.channel_client = Some(ChannelServiceClient::new(channel.clone()));
        self.channel = Some(channel);
        self.connected = true;

        Ok(())
    }
}
