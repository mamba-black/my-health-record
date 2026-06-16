use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt as _;

pub fn init_logger() {
    // 1. Configurar el exportador OTLP de trazas usando Tonic (gRPC)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .expect("Error al construir el exportador OTLP");

    // 2. Crear el recurso del servicio de forma estándar para OTel 0.32
    let resource = Resource::builder()
        .with_service_name("clickcare")
        .build();

    // 3. Crear el SdkTracerProvider usando el SdkTracerProviderBuilder moderno
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // 4. Establecer el proveedor global de telemetría
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("clickcare");

    // 5. Crear el layer de OpenTelemetry para enlazarlo a tracing
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // 6. Crear el formateador para la consola clásica (fmt)
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT | FmtSpan::CLOSE);

    // 7. Unir y registrar todos los layers

    let env_filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug,opentelemetry=info,opentelemetry_sdk=info,h2=info"));
    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(telemetry_layer)
        .with(fmt_layer);

    subscriber.init();
}
