use core::fmt::Display;
use core::str::FromStr;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path::{Path as OsPath, PathBuf};

use super::{Configuration, Ident, ReportDefinition, ReportId};

use super::{ProjectConfig, ReportConfig};

const CONFIGURATION_FILE: &str = ".config/scholium.toml";
const CONFIGURATION_GROUPS_DIR: &str = ".config/scholium";

#[derive(Debug)]
pub enum Error {
    ReadDir(PathBuf, std::io::Error),
    ReadFile(PathBuf, std::io::Error),
    Parse(PathBuf, toml::de::Error),
}

pub fn load_project_config(workspace: &OsPath) -> Result<ProjectConfig, Error> {
    let file = workspace.join(CONFIGURATION_FILE);
    let data = match read_to_string(&file) {
        Ok(data) => data,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(ProjectConfig::default());
        }
        Err(error) => return Err(Error::ReadFile(file, error)),
    };

    toml::from_str(&data).map_err(|source| Error::Parse(file, source))
}

/// Read group information.
pub fn load_groups(workspace: &OsPath) -> Result<Configuration, Error> {
    let folder = workspace.join(CONFIGURATION_GROUPS_DIR);
    if !folder.exists() {
        return Ok(Configuration::default());
    }

    let mut group_display_names = BTreeMap::<Ident, String>::new();
    let mut report_definitions = BTreeMap::<ReportId, ReportDefinition>::new();

    for entry in std::fs::read_dir(&folder)
        .map_err(|source| Error::ReadDir(folder.clone(), source))?
    {
        let entry = entry.map_err(|source| Error::ReadDir(folder.clone(), source))?;
        let path = entry.path();

        // read only files and symlinks.
        if (!path.is_file() && !path.is_symlink())
            || path.extension() != Some(OsStr::new("toml"))
        {
            continue;
        }

        let group = match path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(Ident::from_str)
            .transpose()
        {
            Ok(Some(group_id)) => group_id,
            // Ignore files which file names don't conform local Ident specification
            Ok(None) | Err(_) => {
                continue;
            }
        };

        let data = read_to_string(&path)
            .map_err(|source| Error::ReadFile(path.to_path_buf(), source))?;
        let group_config: ReportConfig = toml::from_str(&data)
            .map_err(|source| Error::Parse(path.to_path_buf(), source))?;

        // filename in folder is unique
        let _ignored =
            group_display_names.insert(group.clone(), group_config.display_name);
        for (report_id, definition) in group_config.reports {
            let path = ReportId::from_parts(group.clone(), report_id);
            let _ignored = report_definitions.insert(path, definition);
        }
    }

    Ok(Configuration {
        group_display_names,
        report_definitions,
    })
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDir(_, source) => Some(source),
            Self::ReadFile(_, source) => Some(source),
            Self::Parse(_, source) => Some(source),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadDir(path, source) => write!(
                f,
                "Unable to list files in folder {}: {source}",
                path.display()
            ),
            Self::ReadFile(path, source) => {
                write!(f, "Unable to read file {}: {source}", path.display())
            }
            Self::Parse(path, source) => {
                write!(f, "Unable to parse file {}: {source}", path.display())
            }
        }
    }
}
