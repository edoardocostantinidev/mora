use log::Level;
use mora_core::result::MoraResult;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::{
    metrics::Temporality,
    propagation::TraceContextPropagator,
    resource::{
        EnvResourceDetector, ResourceDetector, SdkProvidedResourceDetector,
        TelemetryResourceDetector,
    },
    Resource,
};

use std::{env, str::FromStr};

fn get_resource() -> Resource {
    let os_resource = OsResourceDetector;
    let process_resource = ProcessResourceDetector;
    let sdk_resource = SdkProvidedResourceDetector;
    let env_resource = EnvResourceDetector::new();
    let telemetry_resource = TelemetryResourceDetector;

    let resource_detectors: &[Box<dyn ResourceDetector>] = &[
        Box::new(os_resource),
        Box::new(process_resource),
        Box::new(sdk_resource),
        Box::new(env_resource),
        Box::new(telemetry_resource),
    ];

    Resource::builder()
        .with_detectors(resource_detectors)
        .with_attributes([
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("service.instance.id", uuid::Uuid::new_v4().to_string()),
        ])
        .with_service_name("mora-server")
        .build()
}

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .expect("Failed to build tracer provider");

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(get_resource())
        .build();

    global::set_tracer_provider(tracer_provider);
}

fn init_meter_provider() {
    let meter_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_temporality(Temporality::Delta)
        .build()
        .expect("Failed to build meter provider");

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(meter_exporter)
        .with_resource(get_resource())
        .build();

    global::set_meter_provider(meter_provider);
}

fn init_logger_provider() {
    let logger_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .build()
        .expect("Failed to build logger provider");

    let stdout_exporter = opentelemetry_stdout::LogExporter::default();

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(logger_exporter)
        .with_batch_exporter(stdout_exporter)
        .with_resource(get_resource())
        .build();

    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    log::set_boxed_logger(Box::new(otel_log_appender)).unwrap();

    let max_level = env::var("LOG_LEVEL")
        .ok()
        .and_then(|l| Level::from_str(l.to_lowercase().as_str()).ok())
        .unwrap_or(Level::Info);
    log::set_max_level(max_level.to_level_filter());
}

pub fn init_otel() -> MoraResult<()> {
    init_logger_provider();
    init_tracer();
    init_meter_provider();
    Ok(())
}
