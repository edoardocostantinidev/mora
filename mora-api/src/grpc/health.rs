use mora_core::clock::Clock;
use mora_proto::health::{
    health_service_server::HealthService, ClusterStatusData as ProtoClusterStatusData,
    HealthCheckRequest, HealthCheckResponse,
};
use opentelemetry::{
    global,
    trace::{Span, Tracer},
};
use tonic::{Request, Response, Status};

pub struct HealthServiceImpl;

#[tonic::async_trait]
impl HealthService for HealthServiceImpl {
    async fn get_cluster_status(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let mut span = global::tracer("mora-api").start("grpc_health_check");
        log::info!("gRPC Health check endpoint hit");
        let meter = global::meter("mora-api");
        let health_check_endpoint_hits = meter
            .u64_counter("grpc_health_check_endpoint_hits")
            .build();
        health_check_endpoint_hits.add(1, &[]);
        span.set_status(opentelemetry::trace::Status::Ok);
        span.end();

        let current_time_in_ns = Clock::now();
        let response = HealthCheckResponse {
            status: Some(
                mora_proto::health::health_check_response::Status::Online(
                    ProtoClusterStatusData {
                        version: "1.0.0".to_string(),
                        current_time_in_ns: current_time_in_ns.to_le_bytes().to_vec(),
                    },
                ),
            ),
        };

        Ok(Response::new(response))
    }
}
