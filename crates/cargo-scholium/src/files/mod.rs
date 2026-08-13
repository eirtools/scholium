mod ident;
mod read;
mod structs;
#[cfg(test)]
mod tests;

pub use read::{Error as ReadError, load_groups, load_project_config};
pub use structs::{
    Configuration, Profile as ProjectProfile, ProjectConfig, ReportConfig,
    ReportDefinition,
};

pub use ident::{Ident, Prefix, ReportId, ReportIdError, Severity};

#[cfg(test)]
use ident::{Context, IdentError, PrefixError};
