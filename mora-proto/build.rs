fn main() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path("src/descriptor.bin")
        .include_file("descriptor.rs")
        .compile_protos(
            &[
                "../protos/mora/health/v1/health.proto",
                "../protos/mora/queues/v1/queues.proto",
                "../protos/mora/channels/v1/channels.proto",
                "../protos/mora/events/v1/events.proto",
                "../protos/mora/connections/v1/connections.proto",
            ],
            &["../protos/"],
        )
        .expect("Failed to compile gRPC definitions");
}
