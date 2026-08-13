use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::cli::{Output, SeverityOverrides};
use crate::command::common::setup_tracing;
use crate::command::{CommandError, obtain_profile};
use crate::files::{Prefix, ReportDefinition, ReportId, Severity, load_groups};

/// Resolved information to be used in report.
pub(super) struct Profile {
    /// Default severity for unknown report ids.
    ///
    /// If provided, unknown report ids will be treated as normal output.
    pub unknown: Option<Severity>,

    /// Report definitions ready to display.
    pub definitions: HashMap<ReportId, ReportDefinition>,

    /// Suppressed reports via cli or profile.
    pub suppressed: HashSet<ReportId>,
}

pub(super) struct Reports {
    definitions: HashMap<ReportId, ReportDefinition>,
    suppressed: HashSet<ReportId>,
    unknown: Option<Severity>,
}

pub(super) fn resolve_profile_information(
    workspace_root: &Path,
    profile_id: Option<&str>,
    cli_output: Output,
    overrides: SeverityOverrides,
) -> Result<Profile, CommandError> {
    let profile = obtain_profile(workspace_root, profile_id)?;
    let reports =
        obtain_report_definitions(workspace_root, profile.overrides, overrides)?;

    setup_tracing(
        cli_output.format,
        cli_output.detail,
        cli_output.output.as_deref(),
        profile.format,
        profile.detail,
        profile.output.as_deref(),
    )?;

    Ok(Profile {
        unknown: reports.unknown.or(profile.unknown),
        definitions: reports.definitions,
        suppressed: reports.suppressed,
    })
}

#[inline]
fn resolve_cli_severity(
    overrides: SeverityOverrides,
) -> Result<(HashMap<Prefix, Severity>, Option<Severity>), CommandError> {
    #[inline(always)]
    fn fill_overrides(
        severity: Severity,
        prefixes: Vec<Prefix>,
        result: &mut HashMap<Prefix, Severity>,
    ) -> Result<(), CommandError> {
        for prefix in prefixes {
            if result.contains_key(&prefix) {
                return Err(CommandError::CliDuplicatePrefix(prefix));
            }

            result.insert(prefix, severity);
        }
        Ok(())
    }

    let mut result = HashMap::new();
    fill_overrides(Severity::Suppress, overrides.suppress, &mut result)?;
    fill_overrides(Severity::Error, overrides.error, &mut result)?;
    fill_overrides(Severity::Warning, overrides.warning, &mut result)?;
    fill_overrides(Severity::Info, overrides.info, &mut result)?;
    fill_overrides(Severity::Debug, overrides.debug, &mut result)?;
    fill_overrides(Severity::Trace, overrides.trace, &mut result)?;

    Ok((result, overrides.unknown))
}

fn obtain_report_definitions(
    workspace_root: &Path,
    mut overrides: HashMap<Prefix, Severity>,
    cli_severity_overrides: SeverityOverrides,
) -> Result<Reports, CommandError> {
    let (cli_overrides, unknown) = resolve_cli_severity(cli_severity_overrides)?;

    for (prefix, severity) in cli_overrides {
        overrides.insert(prefix, severity);
    }

    let originals = load_groups(workspace_root)?.report_definitions;
    let mut definitions = HashMap::new();
    let mut suppressed = HashSet::new();

    for (path, mut definition) in originals {
        if update_severity(&path, &mut definition, &overrides) {
            suppressed.insert(path);
        } else {
            definitions.insert(path, definition);
        }
    }

    Ok(Reports {
        definitions,
        suppressed,
        unknown,
    })
}

/// Update definition severity and return suppressed status.
#[inline]
fn update_severity(
    path: &ReportId,
    definition: &mut ReportDefinition,
    overrides: &HashMap<Prefix, Severity>,
) -> bool {
    let mut best_match: Option<&Prefix> = None;
    let mut result_severity = definition.severity;

    for (prefix, severity) in overrides {
        if path.starts_with(prefix) && best_match < Some(prefix) {
            best_match = Some(prefix);
            result_severity = *severity;
        }
    }
    let suppressed = matches!(result_severity, Severity::Suppress)
        && definition.severity != result_severity;
    definition.severity = result_severity;
    suppressed
}
