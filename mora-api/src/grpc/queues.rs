use crate::QueuePoolState;
use log::{debug, error};
use mora_proto::queues::{
    queue_service_server::QueueService, CreateQueueRequest, CreateQueueResponse,
    DeleteQueueRequest, DeleteQueueResponse, GetQueueRequest, GetQueueResponse, ListQueuesRequest,
    ListQueuesResponse, Queue,
};
use tonic::{Request, Response, Status};

pub struct QueueServiceImpl {
    pub queue_pool: QueuePoolState,
}

#[tonic::async_trait]
impl QueueService for QueueServiceImpl {
    async fn list_queues(
        &self,
        _request: Request<ListQueuesRequest>,
    ) -> Result<Response<ListQueuesResponse>, Status> {
        debug!("gRPC Received list_queues request");

        let queues: Vec<Queue> = self
            .queue_pool
            .lock()
            .await
            .get_queues(regex::Regex::new(r".*").unwrap())
            .map_err(|e| Status::internal(e.to_string()))?
            .iter()
            .map(|q| Queue {
                id: q.0.to_owned(),
                pending_events_count: q.1.len as u64,
            })
            .collect();

        Ok(Response::new(ListQueuesResponse { queues }))
    }

    async fn get_queue(
        &self,
        request: Request<GetQueueRequest>,
    ) -> Result<Response<GetQueueResponse>, Status> {
        debug!("gRPC Received get_queue request");
        let queue_id = request.into_inner().queue_id;

        let queue: GetQueueResponse = self
            .queue_pool
            .lock()
            .await
            .get_queues(regex::Regex::new(&queue_id).unwrap())
            .map_err(|e| {
                error!("{e}");
                Status::internal(e.to_string())
            })?
            .iter()
            .map(|q| GetQueueResponse {
                id: q.0.to_owned(),
                pending_events_count: 0,
            })
            .collect::<Vec<GetQueueResponse>>()
            .first()
            .ok_or(Status::not_found("Queue not found"))?
            .clone();

        Ok(Response::new(queue))
    }

    async fn create_queue(
        &self,
        request: Request<CreateQueueRequest>,
    ) -> Result<Response<CreateQueueResponse>, Status> {
        let id = request.into_inner().id;
        debug!("gRPC Received create_queue request: {}", &id);

        self.queue_pool
            .lock()
            .await
            .create_queue(id.to_owned())
            .map_err(|e| {
                error!("{e}");
                Status::internal(e.to_string())
            })?;

        Ok(Response::new(CreateQueueResponse {
            id: id.to_owned(),
            pending_events_count: 0,
        }))
    }

    async fn delete_queue(
        &self,
        request: Request<DeleteQueueRequest>,
    ) -> Result<Response<DeleteQueueResponse>, Status> {
        let queue_id = request.into_inner().queue_id;
        debug!("gRPC Received delete_queue request: {}", &queue_id);

        let deleted_id = self
            .queue_pool
            .lock()
            .await
            .delete_queue(queue_id)
            .map_err(|e| {
                let e_msg = format!("error deleting queue: {:?}", e);
                error!("{e_msg}");
                Status::internal(e_msg)
            })?;

        Ok(Response::new(DeleteQueueResponse {
            message: format!("{} deleted", deleted_id),
        }))
    }
}
