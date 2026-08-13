use std::path::{Path, PathBuf};

use cargo_metadata::{Error as MetadataError, Metadata, MetadataCommand};

use crate::cli::ScholiumArgs;
use crate::files::{ProjectProfile, load_project_config};
use crate::tracing::{
    FieldDetail, FormatKind, OutputTarget, TracingError, init_tracing,
};

use super::CommandError;
use super::list_explain::{explain, list};
use super::report::process as report;

pub fn process(args: ScholiumArgs) {
    let result = match args {
        ScholiumArgs::Report(command) => report(command),
        ScholiumArgs::List(command) => list(command),
        ScholiumArgs::Explain(command) => explain(command),
    };

    if let Err(error) = result {
        tracing::error!("{error}")
    }
}

pub(super) fn obtain_metadata(
    manifest_path: Option<PathBuf>,
) -> Result<Metadata, MetadataError> {
    let mut metadata = MetadataCommand::new();
    metadata.no_deps();
    if let Some(path) = manifest_path {
        metadata.cargo_path(path);
    };

    metadata.exec()
}

pub(super) fn obtain_profile(
    workspace_root: &Path,
    profile_id: Option<&str>,
) -> Result<ProjectProfile, CommandError> {
    let config = load_project_config(workspace_root)?;

    Ok(if let Some(profile_id) = profile_id {
        config.profiles.unwrap_or_default().remove(profile_id) // use `remove` to avoid cloning
    } else {
        None
    }
    .unwrap_or_default())
}

pub(super) fn setup_tracing(
    cli_format: Option<FormatKind>,
    cli_detail: Option<FieldDetail>,
    cli_output: Option<&Path>,
    profile_format: Option<FormatKind>,
    profile_detail: Option<FieldDetail>,
    profile_output: Option<&Path>,
) -> Result<(), TracingError> {
    let format = cli_format.or(profile_format).unwrap_or_default();
    let detail = cli_detail.or(profile_detail).unwrap_or_default();
    let target = cli_output
        .or(profile_output)
        .map_or(OutputTarget::Stderr, OutputTarget::File);

    init_tracing(format, detail, target, None, None, true)
}
