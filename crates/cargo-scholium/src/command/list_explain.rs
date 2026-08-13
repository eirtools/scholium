use std::path::Path;

use crate::cli::{ExplainCommand, ListCommand};
use crate::files::{Ident, load_groups};
use crate::tracing::FieldDetail;

use super::common::setup_tracing;
use super::{CommandError, obtain_metadata, obtain_profile};

pub(super) fn list(command: ListCommand) -> Result<(), CommandError> {
    let metadata = obtain_metadata(command.manifest_path)?;
    let detail = cmd_setup_tracing(
        metadata.workspace_root.as_std_path(),
        command.profile_id.as_deref(),
        command.detail,
    )?;

    let groups = load_groups(metadata.workspace_root.as_std_path())?;

    let definitions = &groups.report_definitions;

    let mut last_group_shown: Option<&Ident> = None;
    for (report_id, definition) in definitions {
        match detail {
            crate::tracing::FieldDetail::Compact => println!("{report_id}"),
            crate::tracing::FieldDetail::Full => {
                if last_group_shown != Some(report_id.group()) {
                    if let Some(group_display) =
                        groups.group_display_names.get(report_id.group())
                    {
                        if last_group_shown.is_some() {
                            println!();
                        }
                        println!("{}: {group_display}", report_id.group());
                    }

                    last_group_shown = Some(report_id.group());
                }
                println!("{report_id}: {}", definition.display_name);
            }
        }
    }

    Ok(())
}

pub(super) fn explain(command: ExplainCommand) -> Result<(), CommandError> {
    let metadata = obtain_metadata(command.manifest_path)?;
    let _ignored =
        cmd_setup_tracing(metadata.workspace_root.as_std_path(), None, None)?;

    let groups = load_groups(metadata.workspace_root.as_std_path())?;

    match groups.report_definitions.get(&command.report_id) {
        None => tracing::error!("Unable to find report id {}", command.report_id),
        Some(definition) => {
            println!(
                "{}\nDefault severity: {}\n\n{}",
                definition.short_message, definition.severity, definition.documentation,
            )
        }
    };
    Ok(())
}

fn cmd_setup_tracing(
    workspace_root: &Path,
    profile_id: Option<&str>,
    cli_detail: Option<FieldDetail>,
) -> Result<FieldDetail, CommandError> {
    let profile = obtain_profile(workspace_root, profile_id)?;

    setup_tracing(None, cli_detail, None, None, profile.detail, None)?;

    Ok(cli_detail.or(profile.detail).unwrap_or_default())
}
