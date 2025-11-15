use crate::ConnectionsState;
use mora_proto::connections::{
    connection_service_server::ConnectionService, ConnectionsInfoResponse,
    GetConnectionsInfoRequest,
};
use opentelemetry::{
    global,
    trace::{Span, Tracer},
};
use tonic::{Request, Response, Status};

pub struct ConnectionServiceImpl {
    pub connections: ConnectionsState,
}

#[tonic::async_trait]
impl ConnectionService for ConnectionServiceImpl {
    async fn get_connections_info(
        &self,
        _request: Request<GetConnectionsInfoRequest>,
    ) -> Result<Response<ConnectionsInfoResponse>, Status> {
        let mut span = global::tracer("mora-api").start("grpc_connections_info");
        log::info!("gRPC Connections info endpoint hit");
        let meter = global::meter("mora-api");
        let connections_info_endpoint_hits = meter
            .u64_counter("grpc_connections_info_endpoint_hits")
            .build();
        let clients_connected = self.connections.lock().await.clients_connected();
        connections_info_endpoint_hits.add(1, &[]);
        span.set_status(opentelemetry::trace::Status::Ok);
        span.end();

        Ok(Response::new(ConnectionsInfoResponse {
            clients_connected: clients_connected as u64,
        }))
    }
}
