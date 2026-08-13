use core::ops::Deref;
use core::str::FromStr;

use serde::Deserialize;
use unicode_ident::{is_xid_continue, is_xid_start};

pub use error::{Context, IdentError, PrefixError, ReportIdError};

/// Simple checked unicode identifier.
///
/// Note: `_` is not allowed at the start position.
#[scholium::mark(r#true::r#false, reason = "why not")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)] // common
#[must_use]
pub struct Ident(String);

/// Full or partial at most 2-segmented path prefix representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)] // common
#[must_use]
pub struct Prefix {
    /// Full group ident.
    group: Ident,

    /// Full or partial report id.
    id: Option<Ident>,
}

/// Full or partial at most 2-segmented path representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)] // common
#[must_use]
pub struct ReportId {
    /// Full group ident.
    group: Ident,

    /// Full report id.
    id: Ident,
}

/// Report severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // common
#[derive(Deserialize)] // serde
#[derive(clap::ValueEnum)] // clap
#[serde(rename_all = "kebab-case")]
#[must_use]
pub enum Severity {
    /// Suppress rule reporting.
    #[serde(alias = "none")]
    #[serde(alias = "disable")]
    #[clap(alias = "S")]
    Suppress,

    /// Report rule with Error log level.
    #[clap(alias = "E")]
    Error,

    /// Report rule with Warning log level.
    #[clap(alias = "W")]
    #[clap(alias = "warn")]
    Warning,

    /// Report rule with Info log level.
    #[clap(alias = "I")]
    Info,

    /// Report rule with Debug log level.
    #[clap(alias = "D")]
    Debug,

    /// Report rule with Trace log level.
    #[clap(alias = "T")]
    Trace,
}

impl Deref for Ident {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ReportId {
    #[inline]
    pub fn group(&self) -> &Ident {
        &self.group
    }

    #[inline]
    pub fn id(&self) -> &Ident {
        &self.id
    }

    #[inline]
    pub fn from_parts(group: Ident, id: Ident) -> Self {
        Self { group, id }
    }

    #[inline]
    pub fn from_parts_raw(group: &str, id: &str) -> Result<Self, ReportIdError> {
        fn parse_ident(value: &str, context: Context) -> Result<Ident, ReportIdError> {
            if value.is_empty() {
                return Err(ReportIdError::Empty(context));
            };

            Ident::from_str(value)
                .map_err(|source| ReportIdError::Ident(context, source))
        }

        Ok(Self {
            group: parse_ident(group, Context::Group)?,
            id: parse_ident(id, Context::Id)?,
        })
    }

    pub fn starts_with(&self, prefix: &Prefix) -> bool {
        if self.group() != prefix.group() {
            return false;
        }

        let Some(id) = prefix.id() else {
            return true;
        };

        self.id().as_ref().starts_with(id.as_ref())
    }
}

impl Prefix {
    #[inline]
    pub fn group(&self) -> &Ident {
        &self.group
    }

    #[inline]
    pub fn id(&self) -> Option<&Ident> {
        self.id.as_ref()
    }

    #[inline]
    pub fn from_parts_raw(group: &str, id: Option<&str>) -> Result<Self, PrefixError> {
        fn parse_ident(value: &str, context: Context) -> Result<Ident, PrefixError> {
            Ident::from_str(value).map_err(|source| PrefixError::Ident(context, source))
        }

        if group.is_empty() {
            return Err(PrefixError::Empty);
        }

        let group = parse_ident(group, Context::Group)?;
        let id = match id {
            None | Some("") => None,
            Some(value) => Some(parse_ident(value, Context::Id)?),
        };

        Ok(Self { group, id })
    }
}

impl FromStr for Ident {
    type Err = IdentError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.strip_prefix("r#").unwrap_or(input);

        let mut chars = input.chars();

        let Some(first) = chars.next() else {
            return Err(IdentError::Empty);
        };

        if first == '_' {
            return Err(IdentError::Underscore(input.to_owned()));
        };

        if !is_xid_start(first) {
            return Err(IdentError::Unicode(input.to_owned()));
        }

        if chars.all(is_xid_continue) {
            Ok(Ident(input.to_owned()))
        } else {
            Err(IdentError::Unicode(input.to_owned()))
        }
    }
}

impl FromStr for ReportId {
    type Err = ReportIdError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (group, report_id) = match input.split_once("::") {
            Some((group, report_id)) => (group, report_id),
            None => (input, ""),
        };

        Self::from_parts_raw(group, report_id)
    }
}

impl FromStr for Prefix {
    type Err = PrefixError;

    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (group, report_id) = match input.split_once("::") {
            Some((group, report_id)) => (group, Some(report_id)),
            None => (input, None),
        };

        Self::from_parts_raw(group, report_id)
    }
}

mod display {
    use core::fmt::{Display, Formatter, Result};

    use super::{Ident, Prefix, ReportId, Severity};

    impl Display for Ident {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.0)
        }
    }

    impl Display for ReportId {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}::{}", self.group, self.id)
        }
    }

    impl Display for Prefix {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match &self.id {
                Some(report_id) => write!(f, "{}::{}", self.group, report_id),
                None => write!(f, "{}", self.group),
            }
        }
    }

    impl Display for Severity {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.write_str(match self {
                Self::Suppress => "suppress",
                Self::Error => "error",
                Self::Warning => "warning",
                Self::Info => "info",
                Self::Debug => "debug",
                Self::Trace => "trace",
            })
        }
    }
}
mod severity_adapter {
    use scholium_core::Severity as CoreSeverity;
    use tracing::Level;

    use super::Severity;

    impl From<CoreSeverity> for Severity {
        #[inline]
        fn from(value: CoreSeverity) -> Self {
            match value {
                CoreSeverity::Error => Self::Error,
                CoreSeverity::Warning => Self::Warning,
                CoreSeverity::Info => Self::Info,
                CoreSeverity::Debug => Self::Debug,
                CoreSeverity::Trace => Self::Trace,
            }
        }
    }

    impl From<Severity> for Option<tracing::Level> {
        fn from(value: Severity) -> Self {
            match value {
                Severity::Suppress => None,
                Severity::Error => Some(Level::ERROR),
                Severity::Warning => Some(Level::WARN),
                Severity::Info => Some(Level::INFO),
                Severity::Debug => Some(Level::DEBUG),
                Severity::Trace => Some(Level::TRACE),
            }
        }
    }
}

mod serde_impl {
    //! Serde deserialization implementation.
    use core::str::FromStr;

    use serde::de::Error as DeError;
    use serde::{Deserialize, Deserializer};

    use super::{Ident, Prefix};

    impl<'de> Deserialize<'de> for Prefix {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            FromStr::from_str(&value).map_err(DeError::custom)
        }
    }

    impl<'de> Deserialize<'de> for Ident {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            FromStr::from_str(&value).map_err(DeError::custom)
        }
    }
}

mod error {
    //! Basic parse error implementation.
    use core::error::Error;
    use core::fmt::Display;

    /// Identifier parse error.
    #[derive(Debug, Clone, PartialEq, Eq)] // common
    pub enum IdentError {
        /// Identifier is an empty string.
        Empty,

        /// Identifier starts with underscore (`_`).
        Underscore(String),

        /// A part is not an ident.
        Unicode(String),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)] // common
    pub enum Context {
        /// Group part.
        Group,

        /// Id part.
        Id,
    }

    /// Report id parse error.
    #[derive(Debug, Clone, PartialEq, Eq)] // common
    pub enum ReportIdError {
        /// Identifier is an empty string.
        Empty(Context),

        /// A part is not an ident.
        Ident(Context, IdentError),
    }

    /// Prefix parse error.
    #[derive(Debug, Clone, PartialEq, Eq)] // common
    pub enum PrefixError {
        /// Whole identifier is empty.
        Empty,

        /// A part is not an ident.
        Ident(Context, IdentError),
    }

    impl Error for IdentError {}
    impl Error for ReportIdError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Ident(_, source) => Some(source),
                Self::Empty(_) => None,
            }
        }
    }
    impl Error for PrefixError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Ident(_, source) => Some(source),
                Self::Empty => None,
            }
        }
    }

    impl Display for IdentError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Empty => f.write_str("identifier cannot be an empty string"),
                Self::Underscore(segment) => {
                    write!(
                        f,
                        "identifier cannot start with an underscore (`_`) character: \
                         {segment:?}"
                    )
                }
                Self::Unicode(segment) => {
                    write!(f, "unable recognise as unicode identifier: {segment:?}")
                }
            }
        }
    }

    impl Display for ReportIdError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Empty(context) => write!(
                    f,
                    "report id must be in form `group::id`. Missing `{context}` part"
                ),
                Self::Ident(context, source) => {
                    write!(f, "Report id `{context}` part: {source}")
                }
            }
        }
    }

    impl Display for PrefixError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Empty => write!(
                    f,
                    "prefix must be in form `group` or `group::id` where both parts \
                     are valid unicode identifiers."
                ),
                Self::Ident(context, source) => {
                    write!(f, "Prefix `{context}` part: {source}")
                }
            }
        }
    }

    impl Display for Context {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Group => "group",
                Self::Id => "id",
            })
        }
    }
}
