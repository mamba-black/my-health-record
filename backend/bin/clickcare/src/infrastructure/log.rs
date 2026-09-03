use std::time::Duration;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::debug;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Registry};

/// Configuración común para los exportadores de OpenTelemetry (OTLP).
#[derive(Debug, Clone)]
pub struct OtelConfig {
    pub endpoint: String,
    pub protocol: String,
    pub metric_export_interval: Option<Duration>,
    pub resource: Resource,
}

impl OtelConfig {
    pub fn from_env() -> Self {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());
        let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL")
            .unwrap_or_else(|_| "grpc".to_string());
        let metric_export_interval = std::env::var("OTEL_METRIC_EXPORT_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_millis);

        let resource = Resource::builder().with_service_name("clickcare").build();

        Self {
            endpoint,
            protocol,
            metric_export_interval,
            resource,
        }
    }

    pub fn is_http(&self) -> bool {
        matches!(self.protocol.to_lowercase().as_str(), "http" | "http/protobuf")
    }
}

/// Inicializa todo el sistema de observabilidad (logs de consola, logs OTLP, trazas OTLP y métricas).
pub fn init_observability() {
    let config = OtelConfig::from_env();

    // 1. Inicializar métricas (MeterProvider y System/Process metrics)
    init_meter(&config);

    // 2. Inicializar layer de Tracing OTLP
    let otel_tracing_layer = init_tracing_layer(&config);

    // 3. Inicializar layer de Logs OTLP (nivel INFO hacia adelante)
    let otel_log_layer = init_log_layer(&config);

    // 4. Formateador para la consola clásica (fmt)
    let fmt_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);

    // 5. Filtro global de entorno para la aplicación
    let env_filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("debug,opentelemetry=info,opentelemetry_sdk=info,h2=info,bollard=info")
    });

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(otel_tracing_layer)
        .with(otel_log_layer)
        .with(fmt_layer);

    let _ = subscriber.try_init();

    debug!("OTEL_EXPORTER_OTLP_ENDPOINT: {:?}", config.endpoint);
    debug!("OTEL_EXPORTER_OTLP_PROTOCOL: {:?}", config.protocol);
    debug!("OTEL_METRIC_EXPORT_INTERVAL: {:?}", config.metric_export_interval);
}

/// Configura e inicializa el proveedor de trazas y retorna el layer para `tracing`.
pub fn init_tracing_layer<S>(config: &OtelConfig) -> impl Layer<S>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let span_exporter = if config.is_http() {
        opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de trazas (HTTP)")
    } else {
        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de trazas (gRPC)")
    };

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(config.resource.clone())
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    let tracer = tracer_provider.tracer("clickcare");

    tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Configura e inicializa el proveedor de logs OTLP y retorna el layer filtrado (nivel INFO en adelante).
pub fn init_log_layer<S>(config: &OtelConfig) -> impl Layer<S>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let log_exporter = if config.is_http() {
        opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de logs (HTTP)")
    } else {
        opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de logs (gRPC)")
    };

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(config.resource.clone())
        .build();

    OpenTelemetryTracingBridge::new(&logger_provider).with_filter(LevelFilter::INFO)
}

/// Configura e inicializa el proveedor de métricas global y las métricas de sistema/proceso.
pub fn init_meter(config: &OtelConfig) {
    let metric_exporter = if config.is_http() {
        opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de métricas (HTTP)")
    } else {
        opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(&config.endpoint)
            .build()
            .expect("Error al construir el exportador OTLP de métricas (gRPC)")
    };

    let mut reader_builder = PeriodicReader::builder(metric_exporter);
    if let Some(interval) = config.metric_export_interval {
        reader_builder = reader_builder.with_interval(interval);
    }
    let reader = reader_builder.build();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(config.resource.clone())
        .build();

    opentelemetry::global::set_meter_provider(meter_provider);
    init_meter_system_metrics();

}

pub fn init_meter_system_metrics() {
    let meter = opentelemetry::global::meter("clickcare");

    // --- Métricas del Proceso Actual ---

    // 1. Memoria RAM física real usada por el proceso (Resident Set Size) en Bytes
    let _ = meter
        .u64_observable_gauge("process.resident_memory_bytes")
        .with_description("Memoria RAM física (RSS) usada por el proceso en bytes")
        .with_callback(|observer| {
            if let Some((rss_bytes, _)) = init_meter_system_metrics_process_memory() {
                observer.observe(rss_bytes, &[]);
            }
        })
        .build();

    // 2. Memoria compartida usada por el proceso en Bytes
    let _ = meter
        .u64_observable_gauge("process.shared_memory_bytes")
        .with_description("Memoria compartida (SHR: RssFile + RssShmem) usada por el proceso en bytes")
        .with_callback(|observer| {
            if let Some((_, shared_bytes)) = init_meter_system_metrics_process_memory() {
                observer.observe(shared_bytes, &[]);
            }
        })
        .build();

    // --- Métricas del Host / Sistema Operativo ---

    // 3. Memoria total del sistema en uso (KB)
    let _ = meter
        .u64_observable_gauge("system.memory.used_kb")
        .with_description("Memoria física usada a nivel de sistema operativo en KB")
        .with_callback(|observer| {
            if let Ok(mem) = sys_info::mem_info() {
                observer.observe(mem.total.saturating_sub(mem.avail), &[]);
            }
        })
        .build();

    // 4. Carga de CPU a nivel de sistema (Load Average 1 min)
    let _ = meter
        .f64_observable_gauge("system.cpu.load_average_1m")
        .with_description("Carga promedio de CPU del sistema a 1 minuto")
        .with_callback(|observer| {
            if let Ok(load) = sys_info::loadavg() {
                observer.observe(load.one, &[]);
            }
        })
        .build();
}

/// Obtiene la memoria física (RSS) y compartida (SHR: RssFile + RssShmem) en bytes del proceso actual en Linux.
fn init_meter_system_metrics_process_memory() -> Option<(u64, u64)> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let mut rss_kb = None;
    let mut rss_file_kb = None;
    let mut rss_shmem_kb = None;

    for line in status.lines() {
        if let Some(val) = line.strip_prefix("VmRSS:") {
            rss_kb = val.trim().strip_suffix("kB")?.trim().parse::<u64>().ok();
        } else if let Some(val) = line.strip_prefix("RssFile:") {
            rss_file_kb = val.trim().strip_suffix("kB")?.trim().parse::<u64>().ok();
        } else if let Some(val) = line.strip_prefix("RssShmem:") {
            rss_shmem_kb = val.trim().strip_suffix("kB")?.trim().parse::<u64>().ok();
        }
    }

    let shared_kb = match (rss_file_kb, rss_shmem_kb) {
        (None, None) => None,
        (file, shmem) => Some(file.unwrap_or(0).saturating_add(shmem.unwrap_or(0))),
    };

    Some((rss_kb? * 1024, shared_kb? * 1024))
}
