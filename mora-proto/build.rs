fn main() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["../protos/mora/health/v1/health.proto"], &["../protos/"])
        .expect("Failed to compile gRPC definitions");
}
