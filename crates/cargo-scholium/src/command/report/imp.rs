use std::path::PathBuf;

use indexmap::IndexMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::cli::ReportCommand;

use super::definitions::resolve_profile_information;
use super::error::ProcessError;
use super::fs_walker::resolve_package_files;
use super::log::process_reports;
use super::parse::{Raw, collect_annotations};
use super::{CommandError, obtain_metadata};

pub fn process(command: ReportCommand) -> Result<(), CommandError> {
    let metadata: cargo_metadata::Metadata = obtain_metadata(command.manifest_path)?;
    let workspace_root = metadata.workspace_root.as_std_path();

    let profile = resolve_profile_information(
        workspace_root,
        command.profile_id.as_deref(),
        command.output,
        command.overrides,
    )?;

    let files = resolve_package_files(&metadata, &command.packages)?;

    let parse_results: IndexMap<PathBuf, Result<Raw, ProcessError>> = files
        .into_par_iter()
        .map(|file| {
            let result = std::fs::read_to_string(&file)
                .map_err(ProcessError::Io)
                .and_then(|contents| {
                    collect_annotations(&contents, &profile.suppressed)
                });
            (file, result)
        })
        .collect();

    for (path, result) in parse_results {
        let file = path.strip_prefix(workspace_root).unwrap_or(&path);

        process_reports(
            file,
            result,
            profile.unknown,
            &profile.definitions,
            &profile.suppressed,
            false,
        );
    }

    Ok(())
}
