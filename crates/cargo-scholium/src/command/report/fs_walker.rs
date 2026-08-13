use std::collections::HashSet;
use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, Package, Target, TargetKind};
use indexmap::IndexSet;
use walkdir::WalkDir;

use crate::command::CommandError;

#[scholium::mark(
    scholium::future_question,
    reason = "do we need to output packages in requested package order or it's just a \
              filter?"
)]
pub(super) fn resolve_package_files(
    metadata: &Metadata,
    packages: &[String],
) -> Result<IndexSet<PathBuf>, CommandError> {
    let mut files: IndexSet<PathBuf> = IndexSet::new();
    let mut roots: IndexSet<&Path> = IndexSet::new();

    if packages.is_empty() {
        for package in &metadata.packages {
            collect_root_folders(package, &mut roots);
        }
    } else {
        let package_names: HashSet<&str> =
            packages.iter().map(String::as_str).collect();

        let requested: HashSet<&str> = metadata
            .packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        let unknown: Vec<_> = package_names.difference(&requested).collect();
        if !unknown.is_empty() {
            return Err(CommandError::UnknownPackages(
                unknown
                    .into_iter()
                    .map(|package| (*package).to_owned())
                    .collect(),
            ));
        }

        for package in &metadata.packages {
            if !package_names.contains(&package.name.as_ref()) {
                continue;
            }

            collect_root_folders(package, &mut roots);
        }
    }

    for root in roots {
        files.extend(walk_rs_files(root)?);
    }

    Ok(files)
}

fn collect_root_folders<'a>(package: &'a Package, roots: &mut IndexSet<&'a Path>) {
    let mut collected = vec![];

    for target in &package.targets {
        if is_lib_or_bin(target) {
            if let Some(target_root) = target.src_path.as_std_path().parent() {
                collected.push(target_root)
            }
        }
    }

    let collected = if collected.len() > 1 {
        // amount of targets is presumably small enough, so algorithm is faster than
        // filesystem walking.
        collected.sort_by_key(|p| p.components().count());

        let mut result: Vec<&Path> = Vec::with_capacity(collected.len());

        for root in collected {
            if result.iter().all(|other| !other.starts_with(root)) {
                result.push(root)
            }
        }

        result
    } else {
        collected
    };

    roots.extend(collected);
}

fn is_lib_or_bin(target: &Target) -> bool {
    target
        .kind
        .iter()
        .any(|k| k == &TargetKind::Bin || k == &TargetKind::Lib)
}

fn walk_rs_files(dir: &Path) -> Result<Vec<PathBuf>, CommandError> {
    let mut files = vec![];

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}
