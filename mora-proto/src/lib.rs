pub mod health {
    tonic::include_proto!("mora.health.v1");
}

pub mod queues {
    tonic::include_proto!("mora.queues.v1");
}

pub mod channels {
    tonic::include_proto!("mora.channels.v1");
}

pub mod events {
    tonic::include_proto!("mora.events.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("descriptor.bin");
