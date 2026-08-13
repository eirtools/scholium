//! Command-line interface
use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Args, Parser};

use crate::files::{Prefix, ReportId, Severity};
use crate::tracing::{FieldDetail, FormatKind};

/// Environment variable name for profile.
pub const ENV_PROFILE: &str = "CARGO_SCHOLIUM_PROFILE";

/// parse CLI arguments.
pub fn parse_args() -> ScholiumArgs {
    match CargoArgs::parse() {
        CargoArgs::Scholium { command } => command,
    }
}

/// Style to use for CLI output.
pub(super) const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

/// Fake wrapper to show "cargo" command on top.
// trick is taken from `cargo mutants`.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(clap::Parser)] // clap
#[command(version, name = "cargo", bin_name = "cargo")]
#[command(disable_help_subcommand = true)]
#[command(styles(CLAP_STYLES))]
enum CargoArgs {
    /// Inspect and report `#[scholium::mark(...)]` usage across code.
    Scholium {
        #[clap(subcommand)]
        command: ScholiumArgs,
    },
}

/// Scholium commands.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(clap::Subcommand)] // clap
#[command(
    version,
    about = "Inspect and report `#[scholium::mark(...)]` usage across code."
)]
pub enum ScholiumArgs {
    /// Report `#[scholium::mark(...)]` annotations usage across code.
    Report(ReportCommand),

    /// List known report ids with their meaning.
    List(ListCommand),

    /// Explain known report ids.
    Explain(ExplainCommand),
}

/// List all known report ids.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Args)] // clap
pub struct ListCommand {
    /// Path to Cargo.toml.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    /// Scholium profile to use.
    #[arg(short = 'P', long = "profile", env = ENV_PROFILE)]
    pub profile_id: Option<String>,

    /// Output detail level for human output.
    #[arg(short, long = "detail", value_enum, value_name = "DETAILS")]
    pub detail: Option<FieldDetail>,
}

/// Explain known report ids.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Args)] // clap
pub struct ExplainCommand {
    /// Path to Cargo.toml.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    /// Full report id in form `group::report_id`
    #[arg(value_name = "REPORT_ID")]
    pub report_id: ReportId,
}

/// Report `#[scholium::mark(...)]` annotations usage across code.
#[scholium::mark(scholium::future_imp, reason = "ability to select cargo target(s)")]
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Args)] // clap
pub struct ReportCommand {
    /// Path to Cargo.toml.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    /// Package to report.
    #[arg(short = 'p', long = "package")]
    pub packages: Vec<String>,

    /// Severity overrides.
    #[clap(flatten)]
    pub overrides: SeverityOverrides,

    /// Scholium profile to use.
    #[arg(short = 'P', long = "profile", env = ENV_PROFILE)]
    pub profile_id: Option<String>,

    #[clap(flatten)]
    pub output: Output,
}

const HEADING_OUTPUT: &str = "Output";

/// Output configuration.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Args)] // clap
#[group(required = false, multiple = true)]
pub struct Output {
    /// Output to provided file.
    #[clap(help_heading = HEADING_OUTPUT)]
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format kind.
    #[clap(help_heading = HEADING_OUTPUT)]
    #[arg(short, long = "format", value_enum, value_name = "FORMAT")]
    pub format: Option<FormatKind>,

    /// Output detail level for human output.
    #[clap(help_heading = HEADING_OUTPUT)]
    #[arg(short, long = "detail", value_enum, value_name = "DETAILS")]
    pub detail: Option<FieldDetail>,
}

const HEADING_SEVERITY_OVERRIDES: &str = "Severity overrides";
const VALUE_NAME_SEVERITY: &str = "PREFIX";

#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Args)] // clap
#[group(required = false, multiple = true)]
pub struct SeverityOverrides {
    /// Set severity to `suppress` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'S', value_name = VALUE_NAME_SEVERITY)]
    pub suppress: Vec<Prefix>,

    /// Set severity to `error` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'E', value_name = VALUE_NAME_SEVERITY)]
    pub error: Vec<Prefix>,

    /// Set severity to `warning` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'W', value_name = VALUE_NAME_SEVERITY)]
    pub warning: Vec<Prefix>,

    /// Set severity to `info` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'I', value_name = VALUE_NAME_SEVERITY)]
    pub info: Vec<Prefix>,

    /// Set severity to `debug` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'D', value_name = VALUE_NAME_SEVERITY)]
    pub debug: Vec<Prefix>,

    /// Set severity to `trace` level.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(short = 'T', value_name = VALUE_NAME_SEVERITY)]
    pub trace: Vec<Prefix>,

    /// Default severity for unknown report ids.
    ///
    /// If provided, unknown report ids will be treated as normal output.
    #[clap(help_heading = HEADING_SEVERITY_OVERRIDES)]
    #[arg(long, value_enum, value_name = "SEVERITY")]
    pub unknown: Option<Severity>,
}
