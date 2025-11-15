use crate::QueuePoolState;
use log::debug;
use mora_core::result::MoraError;
use mora_proto::events::{
    event_service_server::EventService, ScheduleEventRequest, ScheduleEventResponse,
};
use tonic::{Request, Response, Status};

pub struct EventServiceImpl {
    pub queue_pool: QueuePoolState,
}

#[tonic::async_trait]
impl EventService for EventServiceImpl {
    async fn schedule_event(
        &self,
        request: Request<ScheduleEventRequest>,
    ) -> Result<Response<ScheduleEventResponse>, Status> {
        debug!("gRPC Received schedule_event request");
        let req = request.into_inner();
        let binary_data = req.data.into_bytes();

        let rule = req.schedule_rule.as_ref().unwrap();
        let queue_name = rule.queue.clone();
        let schedule_for = rule
            .schedule_for
            .parse::<u128>()
            .map_err(|_| Status::invalid_argument("Invalid schedule_for timestamp"))?;

        let mut queue_pool = self.queue_pool.lock().await;
        if let Err(e) = queue_pool.get_queue_mut(&queue_name) {
            if let MoraError::QueueNotFound(..) = e {
                return Err(Status::not_found(format!(
                    "{} queue does not exist",
                    &queue_name
                )));
            } else {
                return Err(Status::internal(e.to_string()));
            }
        }

        queue_pool
            .enqueue(&queue_name, schedule_for, binary_data.clone())
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ScheduleEventResponse {}))
    }
}
