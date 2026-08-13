use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use proc_macro2::LineColumn;

use crate::files::{ReportDefinition, ReportId, ReportIdError, Severity};
use crate::tracing::{
    TARGET_ERROR_MALFORMED_ATTRIBUTE, TARGET_ERROR_PARSE, TARGET_ERROR_UNKNOWN,
    TARGET_REPORT,
};

use super::ProcessError;
use super::parse::Raw;

#[scholium::mark(
    scholium::future_imp,
    reason = "Don't repeat report documentation if met (info and help lines). Decide \
              whether it's per file or global."
)]
#[scholium::mark(
    debug,
    scholium::future_imp,
    reason = "Short & long reasons? Should it be a cut by first sentence or several \
              fields?"
)]
pub fn process_reports(
    file: &Path,
    result: Result<Raw, ProcessError>,
    unknown_severity: Option<Severity>,
    definitions: &HashMap<ReportId, ReportDefinition>,
    suppressed: &HashSet<ReportId>,
    is_long: bool,
) {
    let raw = match result {
        Ok(raw) => raw,
        Err(ProcessError::Io(error)) => {
            tracing::error!("Unable to read file {}: {error}", file.display());
            return;
        }
        Err(ProcessError::Syn(line, location, error)) => {
            report_parse_error(file, &line, location, error);
            return;
        }
        Err(ProcessError::ReportId(location, error)) => {
            // Should never happen.
            // Reported as malformed attribute as it's a discrepancy between
            // `scholium_core` & `cargo scholium`.
            report_report_id_parse(file, location, &error);
            return;
        }
    };

    for (line, location, error) in raw.errors {
        report_malformed_attribute(file, &line, location, error);
    }

    for item in raw.reports {
        let Some(definition) = definitions.get(&item.report_id) else {
            if let Some(severity) = unknown_severity {
                report_unknown(
                    file,
                    item.location,
                    &item.report_id,
                    report_severity(
                        suppressed.contains(&item.report_id),
                        item.severity,
                        severity,
                    ),
                    item.reason,
                    item.see_also,
                );
            } else {
                report_unknown_not_found(file, item.location, &item.report_id);
            };
            continue;
        };

        let info = if is_long {
            definition
                .long_message
                .as_ref()
                .unwrap_or(&definition.short_message)
        } else {
            &definition.short_message
        };

        report_found(
            file,
            item.location,
            &item.report_id,
            report_severity(
                suppressed.contains(&item.report_id),
                item.severity,
                definition.severity,
            ),
            item.reason,
            &definition.display_name,
            info,
            item.see_also,
        );
    }
}

/// Calculate severity to report.
///
/// Return suppressed if default is overridden via CLI or profile.
/// Otherwise allow to override severity in code.
///
/// e.g.
/// * if definition is suppress, but code is warn, use warn.
/// * if definition is info, in code severity is warn, but it's suppressed via profile, return suppress.
fn report_severity(
    suppressed: bool,
    item: Option<Severity>,
    definition: Severity,
) -> Severity {
    if suppressed {
        Severity::Suppress
    } else {
        item.unwrap_or(definition)
    }
}

fn report_parse_error(
    file: &Path,
    line: &str,
    location: LineColumn,
    error: syn::Error,
) {
    println!("x2 {location:#?}");
    tracing::error!(
        target: TARGET_ERROR_PARSE,
        file = %file.display(),
        line = location.line,
        column = location.column,
        source_line = %line,
        error = %error,
        message = %"Syntax error"
    );
}

fn report_malformed_attribute(
    file: &Path,
    line: &str,
    location: LineColumn,
    error: syn::Error,
) {
    tracing::error!(
        target: TARGET_ERROR_MALFORMED_ATTRIBUTE,
        file = %file.display(),
        line = location.line,
        column = location.column,
        source_line = %line,
        error = %error,
        message = %"Malformed `scholium::mark` attribute"
    );
}

// Should never happen.
// Reported as malformed attribute as it's a discrepancy between `scholium_core` &
// `cargo scholium`.
fn report_report_id_parse(file: &Path, location: LineColumn, error: &ReportIdError) {
    tracing::error!(
        target: TARGET_ERROR_MALFORMED_ATTRIBUTE,
        file = %file.display(),
        line = location.line,
        column = location.column,
        error = %error,
        message = %"Malformed `scholium::mark` attribute"
    );
}

fn report_unknown_not_found(file: &Path, location: LineColumn, report_id: &ReportId) {
    tracing::warn!(
        target: TARGET_ERROR_UNKNOWN,
        file = %file.display(),
        line = location.line,
        column = location.column,
        report_id = %report_id,
        "Report id `{report_id}` is unknown"
    )
}

#[scholium::mark(
    third_party::tracing_missing,
    see_also = "https://github.com/tokio-rs/tracing/issues/3585",
    see_also = "https://github.com/tokio-rs/tracing/issues/3593",
    reason = "Collapse match into a single log call."
)]
#[scholium::mark(scholium::future_imp, reason = "Make serde_json lazy")]
fn report_unknown(
    file_path: &Path,
    location: LineColumn,
    report_id: &ReportId,
    severity: Severity,
    reason: Arc<String>,
    see_also: Arc<Vec<String>>,
) {
    let see_also_str = serde_json::to_string(&*see_also)
        .expect("Vec<String> must be serializable to json");

    match severity {
        Severity::Suppress => {}
        Severity::Error => tracing::error!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            reason = %reason,
            message = %report_id,
            see_also = %see_also_str,
        ),
        Severity::Warning => tracing::warn!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            reason = %reason,
            message = %report_id,
            see_also = %see_also_str,
        ),
        Severity::Info => tracing::info!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            reason = %reason,
            message = %report_id,
            see_also = %see_also_str,
        ),
        Severity::Debug => tracing::debug!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            reason = %reason,
            message = %report_id,
            see_also = %see_also_str,
        ),
        Severity::Trace => tracing::trace!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            reason = %reason,
            message = %report_id,
            see_also = %see_also_str,
        ),
    }
}

#[expect(clippy::too_many_arguments, reason = "all fields are from different sources.")]
#[scholium::mark(
    third_party::tracing_missing,
    see_also = "https://github.com/tokio-rs/tracing/issues/3585",
    see_also = "https://github.com/tokio-rs/tracing/issues/3593",
    reason = "Collapse match into a single log call."
)]
#[scholium::mark(scholium::future_imp, reason = "Make serde_json lazy")]
fn report_found(
    file_path: &Path,
    location: LineColumn,
    report_id: &ReportId,
    severity: Severity,
    reason: Arc<String>,
    display_name: &str,
    info: &str,
    see_also: Arc<Vec<String>>,
) {
    let see_also_str = serde_json::to_string(&*see_also)
        .expect("Vec<String> must be serializable to json");
    match severity {
        Severity::Suppress => {}
        Severity::Error => tracing::error!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            report_id = %report_id,
            reason = %reason,
            info = %info,
            message = %display_name,
            see_also = %see_also_str,
        ),
        Severity::Warning => tracing::warn!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            report_id = %report_id,
            reason = %reason,
            info = %info,
            message = %display_name,
            see_also = %see_also_str,
        ),
        Severity::Info => tracing::info!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            report_id = %report_id,
            reason = %reason,
            info = %info,
            message = %display_name,
            see_also = %see_also_str,
        ),
        Severity::Debug => tracing::debug!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            report_id = %report_id,
            reason = %reason,
            info = %info,
            message = %display_name,
            see_also = %see_also_str,
        ),
        Severity::Trace => tracing::trace!(
            target: TARGET_REPORT,
            file = %file_path.display(),
            line = location.line,
            column = location.column,
            report_id = %report_id,
            reason = %reason,
            info = %info,
            message = %display_name,
            see_also = %see_also_str,
        ),
    }
}
