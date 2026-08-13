use core::fmt::Display;

use cargo_metadata::Error as MetadataError;

use crate::files::{Prefix, ReadError};
use crate::tracing::TracingError;

#[derive(Debug)]
pub enum CommandError {
    /// Tracing initialization error.
    Tracing(TracingError),

    /// Cargo metadata resolve error.
    Metadata(MetadataError),

    /// Read configuration error.
    ReadConfigError(ReadError),

    /// Duplicate prefix in CLI.
    CliDuplicatePrefix(Prefix),

    /// Filesystem walking error
    WalkError(walkdir::Error),

    /// Requested packages are unknown.
    UnknownPackages(Vec<String>),
}

impl From<MetadataError> for CommandError {
    fn from(value: MetadataError) -> Self {
        Self::Metadata(value)
    }
}

impl From<ReadError> for CommandError {
    fn from(value: ReadError) -> Self {
        Self::ReadConfigError(value)
    }
}
impl From<walkdir::Error> for CommandError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkError(value)
    }
}

impl From<TracingError> for CommandError {
    fn from(value: TracingError) -> Self {
        Self::Tracing(value)
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Metadata(source) => {
                write!(f, "unable to resolve cargo metadata: {source}")
            }
            Self::Tracing(source) => {
                write!(f, "unable to initialize tracing: {source}")
            }
            Self::ReadConfigError(source) => {
                write!(f, "unable to read scholium configuration: {source}")
            }
            Self::CliDuplicatePrefix(prefix) => {
                write!(f, "duplicate prefix is defined in CLI parameters: {prefix}")
            }
            Self::WalkError(source) => write!(f, "file walk error: {source}"),
            Self::UnknownPackages(packages) => match packages.as_slice() {
                [] => f.write_str("some packages are unknown"),
                [item] => write!(f, "package `{item}` is unknown"),
                _ => {
                    write!(f, "unknown packages: ")?;

                    for (i, name) in packages.iter().enumerate() {
                        match i {
                            0 => write!(f, "`{name}`")?,
                            i if i == packages.len() - 1 => {
                                write!(f, ", and `{name}`")?
                            }
                            _ => write!(f, ", `{name}`")?,
                        }
                    }

                    Ok(())
                }
            },
        }
    }
}
impl core::error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metadata(source) => Some(source),
            Self::ReadConfigError(source) => Some(source),
            Self::WalkError(source) => Some(source),
            Self::Tracing(source) => Some(source),
            Self::CliDuplicatePrefix(_) | Self::UnknownPackages(_) => None,
        }
    }
}
