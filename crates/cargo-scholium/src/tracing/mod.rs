mod colors;
mod error;
mod format_results;
mod format_source_error;
mod init;

pub use error::TracingError;
use format_results::ResultFormatter;
use format_source_error::SourceErrorFormatter;

pub use init::{OutputTarget, init_tracing};
use serde::Deserialize;

pub const TARGET_ERROR_PARSE: &str = "parse_error";
pub const TARGET_ERROR_MALFORMED_ATTRIBUTE: &str = "malformed_attribute";
pub const TARGET_ERROR_UNKNOWN: &str = "unknown_report";
pub const TARGET_REPORT: &str = "report";

/// Environment to use to configure tracing log levels.
const ENV_LOG_CONFIG: &str = "CARGO_SCHOLIUM_LOG";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[derive(clap::ValueEnum)] // clap
pub enum FieldDetail {
    /// Full multiline details for human output.
    #[clap(alias = "f")]
    #[default]
    Full,

    /// Compact one-line details for human output.
    #[clap(alias = "c")]
    Compact,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[derive(clap::ValueEnum)] // clap
pub enum FormatKind {
    /// Human textual output.
    #[default]
    #[clap(alias = "h")]
    Human,

    /// Per-record JSON output.
    #[clap(alias = "j")]
    Json,
}
