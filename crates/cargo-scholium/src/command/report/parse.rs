use std::collections::HashSet;

use alloc::sync::Arc;

use proc_macro2::LineColumn;
use syn::visit::{Visit, visit_attribute};

use scholium_core::Arguments;

use crate::files::{ReportId, Severity};

use super::ProcessError;

const EXPECTED_IDENTS: &[&str] = &["scholium", "mark"];

#[derive(Debug)]
#[must_use]
pub struct Raw {
    pub reports: Vec<RawItem>,
    pub errors: Vec<(String, LineColumn, syn::Error)>,
}

#[derive(Debug)]
pub struct RawItem {
    /// User-defined severity.
    pub severity: Option<Severity>,

    /// Annotation paths split by parts.
    pub report_id: ReportId,

    pub location: LineColumn,

    pub see_also: Arc<Vec<String>>,

    /// User-defined reason why this annotation is located here.
    pub reason: Arc<String>,
}

pub fn collect_annotations(
    content: &str,
    suppressed: &HashSet<ReportId>,
) -> Result<Raw, ProcessError> {
    let file: syn::File = match syn::parse_file(content) {
        Ok(file) => file,
        Err(error) => {
            return Err(ProcessError::Syn(
                content_line(content, &error),
                error.span().start(), // span is thread local
                error,
            ));
        }
    };

    let mut visitor = Visitor::default();
    visitor.visit_file(&file);

    let errors = visitor
        .errors
        .into_iter()
        .map(|error| (content_line(content, &error), error.span().start(), error))
        .collect();

    let result: Result<Vec<RawItem>, ProcessError> = visitor
        .reports
        .into_iter()
        .flat_map(|arguments| {
            let reason = Arc::new(arguments.reason);
            let see_also = Arc::new(arguments.see_also);
            let severity = arguments.severity.map(Severity::from);

            arguments.report_ids.into_iter().filter_map(move |path| {
                let report_id =
                    match ReportId::from_parts_raw(&path.group, &path.report_id) {
                        Ok(report_id) => report_id,
                        Err(error) => {
                            return Some(Err(ProcessError::ReportId(
                                path.span.start(),
                                error,
                            )));
                        }
                    };

                if suppressed.contains(&report_id) {
                    None
                } else {
                    Some(Ok(RawItem {
                        severity,
                        location: path.span.start(),
                        report_id,
                        see_also: Arc::clone(&see_also),
                        reason: Arc::clone(&reason),
                    }))
                }
            })
        })
        .collect();

    let reports = result?;

    Ok(Raw {
        reports,
        errors,
    })
}

fn content_line(content: &str, error: &syn::Error) -> String {
    let location = error.span().start();
    println!("x1: {location:#?}");

    content
        .lines()
        .nth(location.line - 1)
        .unwrap_or("")
        .to_owned()
}

#[derive(Debug, Default)]
struct Visitor {
    reports: Vec<Arguments>,
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_attribute(&mut self, i: &'ast syn::Attribute) {
        if path_is(i.path(), EXPECTED_IDENTS) {
            match i.parse_args() {
                Err(error) => self.errors.push(error),
                Ok(arguments) => self.reports.push(arguments),
            }
        }

        visit_attribute(self, i);
    }
}

fn path_is(path: &syn::Path, idents: &[&str]) -> bool {
    path.segments.iter().map(|ps| &ps.ident).eq(idents.iter())
}
