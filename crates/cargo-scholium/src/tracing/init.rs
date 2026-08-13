use alloc::sync::Arc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use tracing::Level;
use tracing_core::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::{MakeWriter, format as tracing_format, layer};
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};
use tracing_subscriber::{EnvFilter, Layer, Registry};

use super::{
    ENV_LOG_CONFIG, FieldDetail, FormatKind, ResultFormatter, SourceErrorFormatter,
    TARGET_ERROR_MALFORMED_ATTRIBUTE, TARGET_ERROR_PARSE, TARGET_ERROR_UNKNOWN,
    TARGET_REPORT, TracingError,
};

pub enum OutputTarget<'path> {
    Stderr,
    File(&'path Path),
}

#[derive(Clone)]
struct SharedWriter {
    inner: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.inner
            .lock()
            .expect("log writer lock poisoned")
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().expect("log writer lock poisoned").flush()
    }
}

impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Creates a cloneable `MakeWriter` for the given target.
fn create_writer(target: OutputTarget) -> Result<SharedWriter, TracingError> {
    let boxed: Box<dyn Write + Send + Sync> = match target {
        OutputTarget::Stderr => Box::new(std::io::stderr()),
        OutputTarget::File(path) => {
            let file = OpenOptions::new().create(true).append(true).open(path)?;
            Box::new(file)
        }
    };

    Ok(SharedWriter {
        inner: Arc::new(Mutex::new(boxed)),
    })
}

#[scholium::mark(
    scholium::future_imp,
    reason = "Custom Json formatter to support any custom field extractor"
)]
pub fn init_tracing(
    format: FormatKind,
    detail: FieldDetail,
    target: OutputTarget,
    extra_filter: Option<&str>,
    other_level: Option<Level>,
    with_ansi: bool,
) -> Result<(), TracingError> {
    let filter = if let Some(extra) = extra_filter {
        EnvFilter::builder().parse_lossy(extra)
    } else {
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .with_env_var(ENV_LOG_CONFIG)
            .from_env_lossy()
    };

    let writer = create_writer(target)?;

    let registry = Registry::default().with(filter);

    match format {
        FormatKind::Json => {
            let json = layer().json().with_ansi(with_ansi).with_writer(writer);

            registry.with(json).try_init()?
        }
        FormatKind::Human => {
            initialize_human_output(registry, with_ansi, detail, other_level, writer)?
        }
    };

    Ok(())
}

#[scholium::mark(
    third_party::tracing_missing,
    scholium::future_imp,
    see_also = "https://github.com/tokio-rs/tracing/issues/3595",
    see_also = "https://github.com/tokio-rs/tracing/pull/3596",
    reason = "Requested write ansi colors for custom `FormatEvent` & `Layer` \
              implementation to make output more clippy-like"
)]
fn initialize_human_output(
    registry: Layered<EnvFilter, Registry>,
    with_ansi: bool,
    detail: FieldDetail,
    other_level: Option<Level>,
    writer: SharedWriter,
) -> Result<(), TryInitError> {
    let formatter = tracing_format()
        .compact()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_source_location(false)
        .with_line_number(false)
        .with_file(false)
        .without_time();

    let layer_parse_error = {
        let filter = Targets::new()
            .with_default(LevelFilter::OFF)
            .with_target(TARGET_ERROR_PARSE, Level::TRACE) // level is ignored when used only for routing
            .with_target(TARGET_ERROR_MALFORMED_ATTRIBUTE, Level::TRACE)
            .with_target(TARGET_ERROR_UNKNOWN, other_level.unwrap_or(Level::TRACE));

        layer()
            .event_format(formatter.clone())
            .with_ansi(with_ansi)
            .with_ansi_sanitization(true)
            .fmt_fields(SourceErrorFormatter::new(detail))
            .with_writer(writer.clone())
            .with_filter(filter)
    };

    let layer_results = {
        let filter = Targets::new()
            .with_default(LevelFilter::OFF)
            .with_target(TARGET_REPORT, other_level.unwrap_or(Level::TRACE));

        layer()
            .event_format(formatter.clone())
            .with_ansi(with_ansi)
            .with_ansi_sanitization(true)
            .fmt_fields(ResultFormatter::new(detail))
            .with_writer(writer.clone())
            .with_filter(filter)
    };

    let layer_default = {
        let filter = Targets::new()
            .with_default(LevelFilter::TRACE)
            .with_target(TARGET_ERROR_PARSE, LevelFilter::OFF)
            .with_target(TARGET_ERROR_MALFORMED_ATTRIBUTE, LevelFilter::OFF)
            .with_target(TARGET_ERROR_UNKNOWN, LevelFilter::OFF)
            .with_target(TARGET_REPORT, LevelFilter::OFF);

        layer()
            .event_format(formatter)
            .with_ansi(with_ansi)
            .with_ansi_sanitization(true)
            .with_writer(writer)
            .with_filter(filter)
    };

    registry
        .with(layer_parse_error)
        .with(layer_results)
        .with(layer_default)
        .try_init()
}
