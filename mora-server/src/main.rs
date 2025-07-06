use std::{sync::Arc, time::Duration};

use crate::config::MoraConfig;
use mora_api::MoraApi;
use mora_channel::ChannelManager;
use mora_core::result::MoraResult;
use mora_queue::pool::QueuePool;
use opentelemetry::global;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol, SpanExporter};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use std::sync::OnceLock;
use tokio::{sync::Mutex, task::JoinSet, time::sleep};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

pub mod config;

#[derive(Debug)]
pub struct Server {
    config: MoraConfig,
}

impl Server {
    pub fn new(config: MoraConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> MoraResult<()> {
        let mut tasks = JoinSet::new();
        let channel_manager = Arc::new(Mutex::new(ChannelManager::new()));
        let queue_pool = Arc::new(Mutex::new(QueuePool::new(
            self.config.queue_pool_capacity(),
        )));
        let api = MoraApi::new(self.config.port());
        let channel_manager_for_api = channel_manager.clone();
        let queue_pool_for_api = queue_pool.clone();
        tasks.spawn(async move {
            api.start_listening(channel_manager_for_api, queue_pool_for_api)
                .await
        });

        let channel_manager_for_checker = channel_manager.clone();
        tasks.spawn(async move {
            loop {
                sleep(Duration::from_millis(1)).await;
                let mut binding = channel_manager_for_checker.lock().await;

                binding
                    .get_mut_channels()?
                    .into_iter()
                    .for_each(|channel| channel.update_msec_from_last_op(1));

                let channels_to_delete: Vec<String> = binding
                    .get_channels()?
                    .into_iter()
                    .filter(|c| c.msec_from_last_op() > self.config.channel_timeout_in_msec())
                    .map(|c| c.id().to_owned())
                    .collect();
                if !channels_to_delete.is_empty() {
                    info!(
                        "Will close {} active channels due to inactivity.",
                        channels_to_delete.len()
                    )
                }
                for channel_id in channels_to_delete {
                    binding.close_channel(&channel_id.to_owned());
                }
            }
        });

        while let Some(_) = tasks.join_next().await {
            info!("Tasks completed");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> MoraResult<()> {
    let config = MoraConfig::build()?;

    let logger_provider = init_logs();
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let filter_otel = EnvFilter::new(config.log_level().to_string());
    let otel_layer = otel_layer.with_filter(filter_otel);
    let filter_fmt = EnvFilter::new(config.log_level().to_string());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_target(false)
        .with_line_number(false)
        .with_file(false)
        .compact()
        .with_filter(filter_fmt);

    let tracer_provider = init_traces();
    global::set_tracer_provider(tracer_provider);

    let meter_provider = init_metrics();
    global::set_meter_provider(meter_provider);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    let server = Server::new(config);
    server.run().await
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name("mora-server").build())
        .clone()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_traces() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}
