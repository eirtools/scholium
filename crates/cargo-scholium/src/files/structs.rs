use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use serde::Deserialize;

use super::{Ident, Prefix, ReportId, Severity};
use crate::tracing::{FieldDetail, FormatKind};

#[derive(Debug, Clone, Default, PartialEq, Eq)] // common
pub struct Configuration {
    pub group_display_names: BTreeMap<Ident, String>,
    pub report_definitions: BTreeMap<ReportId, ReportDefinition>,
}

/// Full report description.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[serde(rename_all = "kebab-case")]
pub struct ReportDefinition {
    /// Default report severity.
    pub severity: Severity,

    /// Report display name.
    pub display_name: String,

    /// Concise description used in normal output.
    pub short_message: String,

    /// Long description used in normal output if requested.
    ///
    /// `short_message` is used by default.
    pub long_message: Option<String>,

    /// Full explanation and motivation used by `cargo scholium explain`.
    pub documentation: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[serde(rename_all = "kebab-case")]
#[must_use]
pub struct ProjectConfig {
    pub profiles: Option<HashMap<String, Profile>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[serde(rename_all = "kebab-case")]
#[must_use]
pub struct Profile {
    /// Output format kind.
    pub format: Option<FormatKind>,

    /// Output details for human output.
    pub detail: Option<FieldDetail>,

    /// Default severity for unknown report ids.
    ///
    /// If provided, unknown report ids will be treated as normal output.
    pub unknown: Option<Severity>,

    /// Output to provided file.
    pub output: Option<PathBuf>,

    /// Severity overrides.
    pub overrides: HashMap<Prefix, Severity>,
}

/// Report configuration file.
#[derive(Debug, Clone, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[serde(rename_all = "kebab-case")]
#[must_use]
pub struct ReportConfig {
    /// Display name for help.
    pub display_name: String,

    /// report definitions.
    #[serde(rename = "report")]
    pub reports: HashMap<Ident, ReportDefinition>,
}
