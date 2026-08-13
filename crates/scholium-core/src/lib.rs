//! Parse and check consistency of `scholium::mark` attribute.
//!
//! ## Feature flags
//!
//! `with-details`: function actually collect and return values instead of discarding
//! them after parsing.
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;
mod parse;
#[cfg(test)]
mod tests;

#[cfg(feature = "with-details")]
pub use parse::ReportId;
pub use parse::{Arguments, Severity, parse_mark_attrs};
