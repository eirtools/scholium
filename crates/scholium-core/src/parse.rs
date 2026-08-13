#[cfg(feature = "with-details")]
use alloc::string::String;
use alloc::string::ToString as _;
#[cfg(feature = "with-details")]
use alloc::vec::Vec;

#[cfg(feature = "with-details")]
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
#[cfg(feature = "with-details")]
use syn::spanned::Spanned;

/// Parsed and checked annotation arguments.
#[derive(Debug, Clone)]
#[must_use]
pub struct Arguments {
    /// User-defined severity.
    #[cfg(feature = "with-details")]
    pub severity: Option<Severity>,

    /// Annotation report-ids split by parts.
    #[cfg(feature = "with-details")]
    pub report_ids: Vec<ReportId>,

    /// Annotation see_also field values.
    #[cfg(feature = "with-details")]
    pub see_also: Vec<String>,

    /// User-defined reason why this annotation is located here.
    #[cfg(feature = "with-details")]
    pub reason: String,
}

/// Annotation mark severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Severity {
    /// User reported Error severity level.
    Error,

    /// User reported Warning severity level.
    Warning,

    /// User reported Info severity level.
    Info,

    /// User reported Debug severity level.
    Debug,

    /// User reported Trace severity level.
    Trace,
}

/// User defined full report id.
#[cfg(feature = "with-details")]
#[derive(Debug, Clone)]
#[must_use]
pub struct ReportId {
    /// User defined report group.
    pub group: String,

    /// User defined report id.
    pub report_id: String,

    /// Full report span.
    pub span: Span,
}

/// Parse proc-macro attributes as in form below.
///
/// Notation:
/// * `<severity>, <list of report-ids>, key-value=<lit-str>, reason=<str>`
/// * `<severity>, <list of report-ids>, reason=<str>`
/// * `<list of report-ids>, see_also=<str>, reason=<str>`
/// * `<list of report-ids>, reason=<str>`
///
/// Explanation:
/// * `severity` is lower-case variants of [`Severity`] enum.
/// * `list of report-ids` one or more comma-separated 2-segment Rust Path, which must
///   NOT be absolute.
/// * `see-also` is a user-defined text for see-also. Might be mixed with reason.
/// * `reason` is a user-defined reason why this annotation is located at the element.
// TODO: optimize algorithm. optionally remove syn entirely.
pub fn parse_mark_attrs(tokens: TokenStream) -> syn::Result<Arguments> {
    syn::parse2(tokens)
}

/// Error suggesting order description when violated.
const ERR_ELEMENT_ORDER: &str = "expected order of elements: [<severity>], <list of \
                                 report-ids>, [see_also=<str>,] reason=<str>";

impl Parse for Arguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Easy way to report full range at the end.
        let things_begin = input.cursor();

        let segments =
            Punctuated::<ItemElement, syn::Token![,]>::parse_terminated(input)?;

        // Whether severity was met to avoid double-severity.
        let mut met_severity = false;
        // Whether 2-part path was met.
        let mut met_report_ids = false;
        // Whether any key-value was met.
        let mut met_key_value = false;
        // Whether reason (specific key-value) was met.
        let mut met_reason = false;

        // Collected severity.
        #[cfg(feature = "with-details")]
        let mut severity: Option<Severity> = None;
        // Collected report ids.
        #[cfg(feature = "with-details")]
        let mut report_ids: Vec<ReportId> = Vec::new();
        // Collected reason.
        #[cfg(feature = "with-details")]
        let mut reason = String::new();
        // Collected see-also items.
        #[cfg(feature = "with-details")]
        let mut see_also = Vec::new();

        for segment in &segments {
            match segment {
                ItemElement::Value(ident) => {
                    // Exclude repetition and out-of order.
                    if met_severity || met_report_ids {
                        return Err(syn::Error::new_spanned(ident, ERR_ELEMENT_ORDER));
                    }
                    met_severity = true;

                    #[cfg_attr(
                        not(feature = "with-details"),
                        expect(unused_variables, reason = "used for with-details")
                    )]
                    let severity_value = match Severity::try_from(ident) {
                        Ok(value) => value,
                        Err(()) => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "unable to parse severity",
                            ));
                        }
                    };

                    // could be replaced with `=`, but block will be required
                    #[cfg(feature = "with-details")]
                    severity.replace(severity_value);
                }
                ItemElement::Path(path) => {
                    // Exclude out-of order.
                    if met_key_value {
                        return Err(syn::Error::new_spanned(path, ERR_ELEMENT_ORDER));
                    }
                    met_report_ids = true;

                    // support only 2-parts report-ids.
                    if path.segments.len() == 2 {
                        let group = path.segments[0].ident.to_string();
                        let report_id = path.segments[1].ident.to_string();

                        // For those who want an ability for identifiers to start
                        //   with `_`: prepare an explanation _why_
                        //   these IDs should be included,
                        //   taking in account meaning of this prefix in Rust.
                        if group.starts_with('_') || report_id.starts_with('_') {
                            return Err(syn::Error::new_spanned(
                                path,
                                "report id should not contain segments starting with \
                                 `_`",
                            ));
                        }

                        #[cfg(feature = "with-details")]
                        {
                            report_ids.push(ReportId {
                                group,
                                report_id,
                                span: path.span(),
                            });
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "expected exactly 2 path segments",
                        ));
                    }
                }
                ItemElement::KeyValue(ident, lit) => {
                    #[cfg(not(feature = "with-details"))]
                    let _ignored = lit;

                    // Exclude repetition and out-of order.
                    // Allow mixed see_also and reason encounters.
                    if !met_report_ids {
                        return Err(syn::Error::new_spanned(ident, ERR_ELEMENT_ORDER));
                    }

                    met_key_value = true;

                    if ident == "reason" {
                        // Exclude repetition of `reason`.
                        if met_reason {
                            return Err(syn::Error::new_spanned(
                                ident,
                                ERR_ELEMENT_ORDER,
                            ));
                        }
                        met_reason = true;
                        #[cfg(feature = "with-details")]
                        {
                            reason = lit.value();
                        }
                        continue;
                    }

                    if ident == "see_also" {
                        #[cfg(feature = "with-details")]
                        {
                            see_also.push(lit.value());
                        }
                        continue;
                    }

                    // key is not recognized.
                    return Err(syn::Error::new_spanned(ident, "unknown element"));
                }
            }
        }

        // Check if format was fulfilled.
        if !met_reason {
            let things_end = input.cursor();
            return Err(syn::Error::new_range(
                things_begin..things_end,
                ERR_ELEMENT_ORDER,
            ));
        }

        #[cfg(feature = "with-details")]
        return Ok(Self {
            severity,
            report_ids,
            see_also,
            reason,
        });
        #[cfg(not(feature = "with-details"))]
        return Ok(Self {});
    }
}

/// A single element in attribute sequence.
enum ItemElement {
    /// A single ident.
    Value(syn::Ident),

    /// Segmented path consisting at least 2 elements and not absolute.
    Path(syn::Path),

    /// `key=value` expression where value is a literal str.
    KeyValue(syn::Ident, syn::LitStr),
}

impl Parse for ItemElement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // If second token is path separator, parse it as path.
        if input.peek2(syn::token::PathSep) {
            return Ok(Self::Path(input.parse()?));
        }

        // KeyValue part, assume ident is first token
        if input.peek2(syn::token::Eq) {
            let ident = input.parse()?;
            let _sep: syn::token::Eq = input.parse()?;
            let value = input.parse()?;
            return Ok(Self::KeyValue(ident, value));
        }

        // Try parse it as an ident.
        Ok(Self::Value(input.parse()?))
    }
}

mod aux_impl {
    //! Implementations for Severity and ReportId.

    use core::convert::TryFrom;
    use core::fmt::Display;

    use syn::Ident;

    use super::Severity;

    #[cfg(feature = "with-details")]
    use super::ReportId;

    impl Display for super::Severity {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(match self {
                Self::Error => "error",
                Self::Warning => "warning",
                Self::Info => "info",
                Self::Debug => "debug",
                Self::Trace => "trace",
            })
        }
    }

    impl TryFrom<&Ident> for super::Severity {
        // Proper error is reported by syn, so there's no need to keep context here.
        type Error = ();

        fn try_from(
            value: &Ident,
        ) -> Result<Self, <Severity as TryFrom<&Ident>>::Error> {
            if value == "error" {
                Ok(Self::Error)
            } else if value == "warning" {
                Ok(Self::Warning)
            } else if value == "info" {
                Ok(Self::Info)
            } else if value == "debug" {
                Ok(Self::Debug)
            } else if value == "trace" {
                Ok(Self::Trace)
            } else {
                Err(())
            }
        }
    }

    #[cfg(feature = "with-details")]
    impl Display for ReportId {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}::{}", self.group, self.report_id)
        }
    }
}
