use crate::{ChannelManagerState, QueuePoolState};
use log::{debug, info};
use mora_core::{clock::Clock, result::MoraError};
use mora_proto::channels::{
    channel_service_server::ChannelService, BufferOptions, Channel, CreateChannelRequest,
    CreateChannelResponse, DeleteChannelRequest, DeleteChannelResponse, Event,
    GetChannelEventsRequest, GetChannelEventsResponse, GetChannelRequest, GetChannelResponse,
    ListChannelsRequest, ListChannelsResponse,
};
use tonic::{Request, Response, Status};

pub struct ChannelServiceImpl {
    pub channel_manager: ChannelManagerState,
    pub queue_pool: QueuePoolState,
}

#[tonic::async_trait]
impl ChannelService for ChannelServiceImpl {
    async fn list_channels(
        &self,
        _request: Request<ListChannelsRequest>,
    ) -> Result<Response<ListChannelsResponse>, Status> {
        debug!("gRPC Received list_channels request");

        let channels = self
            .channel_manager
            .lock()
            .await
            .get_channels()
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(|channel| Channel {
                channel_id: channel.id().to_owned(),
                queues: channel.queues().to_owned(),
                buffer_options: Some(BufferOptions {
                    size: channel.buffer_size() as u64,
                    time: channel.buffer_time() as u64,
                }),
                msec_from_last_op: channel.msec_from_last_op() as u64,
            })
            .collect();

        Ok(Response::new(ListChannelsResponse { channels }))
    }

    async fn get_channel(
        &self,
        request: Request<GetChannelRequest>,
    ) -> Result<Response<GetChannelResponse>, Status> {
        let channel_id = request.into_inner().channel_id;
        debug!("gRPC Received get_channel request: {}", &channel_id);

        let mut channel_manager = self.channel_manager.lock().await;
        let channel = channel_manager
            .get_mut_channel(&channel_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        match channel {
            None => Err(Status::not_found(format!(
                "{} channel does not exist",
                &channel_id
            ))),
            Some(channel) => Ok(Response::new(GetChannelResponse {
                channel_id: channel.id().to_owned(),
                queues: channel.queues().to_owned(),
                buffer_options: Some(BufferOptions {
                    size: channel.buffer_size() as u64,
                    time: channel.buffer_time() as u64,
                }),
                msec_from_last_op: channel.msec_from_last_op() as u64,
            })),
        }
    }

    async fn create_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<CreateChannelResponse>, Status> {
        debug!("gRPC Received create_channel request");
        let req = request.into_inner();
        let pool = self.queue_pool.lock().await;
        let buffer_options = req.buffer_options.ok_or(Status::invalid_argument(
            "buffer_options is required",
        ))?;

        let channel = self
            .channel_manager
            .lock()
            .await
            .create_channel(
                &pool,
                req.queues,
                buffer_options.size as usize,
                buffer_options.time as u128,
            )
            .map_err(|e| match e {
                MoraError::QueueNotFound(queue) => {
                    Status::not_found(format!("{} queue does not exist", queue))
                }
                _ => Status::internal(format!("couldn't create channel: {e}")),
            })?;

        debug!("channel created {:?}", &channel);
        Ok(Response::new(CreateChannelResponse {
            channel_id: channel.id().to_owned(),
        }))
    }

    async fn delete_channel(
        &self,
        request: Request<DeleteChannelRequest>,
    ) -> Result<Response<DeleteChannelResponse>, Status> {
        let channel_id = request.into_inner().channel_id;
        debug!("gRPC Received delete_channel request: {}", &channel_id);

        let mut channel_manager = self.channel_manager.lock().await;
        channel_manager.close_channel(&channel_id);

        Ok(Response::new(DeleteChannelResponse {}))
    }

    async fn get_channel_events(
        &self,
        request: Request<GetChannelEventsRequest>,
    ) -> Result<Response<GetChannelEventsResponse>, Status> {
        info!("gRPC Received get_channel_events request");
        let req = request.into_inner();
        let channel_id = req.channel_id;
        let delete = req.delete;

        let mut channel_manager = self.channel_manager.lock().await;
        let mut queue_pool = self.queue_pool.lock().await;
        let channel_opt = channel_manager
            .get_mut_channel(&channel_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        match channel_opt {
            Some(channel) => {
                channel.reset_msec_from_last_op();
                let timestamp = Clock::now();
                let delta = channel.buffer_time();
                let mut events: Vec<Event> = vec![];
                let queues = channel.queues();
                info!("Found {:?}", &queues);

                for queue_name in queues {
                    let data = queue_pool
                        .dequeue_until(queue_name, timestamp + delta, delete)
                        .map_err(|e| Status::internal(e.to_string()))?;
                    info!("Data Found {:?}", &data);

                    let dequeued_events = data
                        .iter()
                        .map(|data| {
                            Ok(Event {
                                timestamp: data.0.to_le_bytes().to_vec(),
                                queue_name: queue_name.to_owned(),
                                data: std::str::from_utf8(&data.1)
                                    .map_err(|e| Status::internal(e.to_string()))?
                                    .to_owned(),
                            })
                        })
                        .collect::<Result<Vec<_>, Status>>()?;
                    events.extend(dequeued_events)
                }

                Ok(Response::new(GetChannelEventsResponse { events }))
            }
            None => Err(Status::not_found(format!(
                "{} channel does not exist",
                &channel_id
            ))),
        }
    }
}
