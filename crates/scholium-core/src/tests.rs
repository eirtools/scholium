use alloc::string::ToString as _;
#[cfg(feature = "with-details")]
use alloc::vec::Vec;
use assert_matches::assert_matches;

use proc_macro2::{LineColumn, TokenStream};
use rstest::rstest;

#[cfg(feature = "with-details")]
use crate::ReportId;

use super::{Arguments, Severity, parse_mark_attrs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestId {
    FullValidError,
    FullValidWarning,
    FullValidInfo,
    FullValidDebug,
    FullValidTrace,
    NoSeverityValid,
    SinglePathValid,
    MultiplePaths,
    SeeAlsoBeforeReason,
    SeeAlsoAfterReason,
    SeeAlsoMixedReason,
    MissingReason,
    OrderPathsBeforeReason,
    OrderSeverityAfterPaths,
    OrderSeeAlsoBeforePaths,
    DuplicateReason,
    ElementAfterReason,
    UnknownSeverityValue,
    WrongKey,
    PathThreeSegments,
    ReasonWithoutPaths,
    DuplicateSeverity,
    SeverityOnly,
    EmptyInput,
    IntLiteral,
    AbsolutePathErr,
    UnderscoreGroup,
    UnderscoreReportId,
}

struct TestCase {
    id: TestId,
    input: &'static str,
    expected: ExpectedResult,
}

enum ExpectedResult {
    Ok(OkExpected),
    Err(ErrExpected),
}

#[cfg_attr(
    not(feature = "with-details"),
    expect(
        dead_code,
        reason = "more work to remove these fields when feature is disabled"
    )
)]
struct OkExpected {
    severity: Option<Severity>,
    report_ids: &'static [(&'static str, &'static str, usize, usize)],
    reason: &'static str,
    see_also: &'static [&'static str],
}

struct ErrExpected {
    message_contains: &'static str,
    span_text: Option<&'static str>,
    // Assume all tests are one-liners, otherwise expected structure need to be
    // adjusted.
    span_col_start: usize,
    span_col_end: usize,
}

#[rstest]
#[case(
TestCase {
    id: TestId::FullValidError,
    input: "error, a::b, c::d, reason=\"42\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: Some(Severity::Error),
        report_ids: &[("a", "b", 7, 11), ("c", "d", 13, 17)],
        see_also: &[],
        reason: "42",
    }),
})]
#[case(
TestCase {
    id: TestId::FullValidWarning,
    input: "warning, a::b, c::d, reason=\"42\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: Some(Severity::Warning),
        report_ids: &[("a", "b", 9, 13), ("c", "d", 15, 19)],
        see_also: &[],
        reason: "42",
    }),
})]
#[case(
TestCase {
    id: TestId::FullValidInfo,
    input: "info, a::b, c::d, reason=\"42\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: Some(Severity::Info),
        report_ids: &[("a", "b", 6, 10), ("c", "d", 12, 16)],
        see_also: &[],
        reason: "42",
    }),
})]
#[case(
TestCase {
    id: TestId::FullValidDebug,
    input: "debug, a::b, c::d, reason=\"\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: Some(Severity::Debug),
        report_ids: &[("a", "b", 7, 11), ("c", "d", 13, 17)],
        see_also: &[],
        reason: "",
    }),
})]
#[case(
TestCase {
    id: TestId::FullValidTrace,
    input: "trace, a::b, c::d, reason=\"42\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: Some(Severity::Trace),
        report_ids: &[("a", "b", 7, 11), ("c", "d", 13, 17)],
        see_also: &[],
        reason: "42",
    }),
})]
#[case(
TestCase {
    id: TestId::NoSeverityValid,
    input: "x::y, p::q, reason=\"0\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("x", "y", 0, 4), ("p", "q", 6, 10)],
        see_also: &[],
        reason: "0",
    }),
})]
#[case(
TestCase{
    id: TestId::SinglePathValid,
    input: "a::b, reason=\"7\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("a", "b", 0, 4)],
        see_also: &[],
        reason: "7",
    }),
})]
#[case(
TestCase{
    id: TestId::MultiplePaths,
    input: "p::q, r::s, t::u, reason=\"12345\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("p", "q", 0, 4), ("r", "s", 6, 10), ("t", "u", 12, 16)],
        see_also: &[],
        reason: "12345",
    }),
})]
#[case(
TestCase{
    id: TestId::SeeAlsoBeforeReason,
    input: "p::q, see_also=\"data\", see_also=\"\", reason=\"12345\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("p", "q", 0, 4)],
        see_also: &["data", ""],
        reason: "12345",
    }),
})]
#[case(
TestCase{
    id: TestId::SeeAlsoAfterReason,
    input: "p::q, reason=\"12345\", see_also=\"data\", see_also=\"\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("p", "q", 0, 4)],
        see_also: &["data", ""],
        reason: "12345",
    }),
})]
#[case(
TestCase{
    id: TestId::SeeAlsoMixedReason,
    input: "p::q, see_also=\"data\", reason=\"12345\", see_also=\"\"",
    expected: ExpectedResult::Ok(OkExpected {
        severity: None,
        report_ids: &[("p", "q", 0, 4)],
        see_also: &["data", ""],
        reason: "12345",
    }),
})]
#[case(
TestCase{
    id: TestId::MissingReason,
    input: "info, a::b",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("info, a::b"),
        span_col_start: 0,
        span_col_end: 10,
    }),
})]
#[case(
TestCase{
    id: TestId::OrderPathsBeforeReason,
    input: "info, reason=\"5\", a::b",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("reason"),
        span_col_start: 6,
        span_col_end: 12,
    }),
})]
#[case(
TestCase{
    id: TestId::OrderSeverityAfterPaths,
    input: "a::b, info, reason=\"5\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("info"),
        span_col_start: 6,
        span_col_end: 10,
}),
})]
#[case(
TestCase{
    id: TestId::OrderSeeAlsoBeforePaths,
    input: "see_also=\"data\", a::b, reason = \"reason\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("see_also"),
        span_col_start: 0,
        span_col_end: 8,
}),
})]
#[case(
TestCase{
    id: TestId::DuplicateReason,
    input: "a::b, reason=\"reason\", reason=\"other\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("reason"),
        span_col_start: 23,
        span_col_end: 29,
}),
})]
#[case(
TestCase{
    id: TestId::ElementAfterReason,
    input: "a::b, reason=\"reason\", info",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("info"),
        span_col_start: 23,
        span_col_end: 27,
}),
})]
#[case(
TestCase{
    id: TestId::UnknownSeverityValue,
    input: "not_exist, a::b, reason=\"reason\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "unable to parse severity",
        span_text: Some("not_exist"),
        span_col_start: 0,
        span_col_end: 9,
    }),
})]
#[case(
TestCase{
    id: TestId::WrongKey,
    input: "a::b, x=\"data\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "unknown element",
        span_text: Some("x"),
        span_col_start: 6,
        span_col_end: 7,
    }),
})]
#[case(
TestCase{
    id: TestId::PathThreeSegments,
    input: "info, a::b::c, reason=\"reason\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected exactly 2 path segments",
        span_text: Some("a::b::c"),
        span_col_start: 6,
        span_col_end: 13,
    }),
})]
#[case(
TestCase{
    id: TestId::ReasonWithoutPaths,
    input: "reason=\"reason\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("reason"),
        span_col_start: 0,
        span_col_end: 6,
    }),
})]
#[case(TestCase {
    id: TestId::DuplicateSeverity,
    input: "info, trace, a::b, reason=\"reason\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("trace"),
        span_col_start: 6,
        span_col_end: 11,
    }),
})]
#[case(TestCase {
    id: TestId::SeverityOnly,
    input: "info",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: Some("info"),
        span_col_start: 0,
        span_col_end: 4,
    }),
})]
#[case(TestCase {
    id: TestId::EmptyInput,
    input: "",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected order of elements",
        span_text: None,
        span_col_start: 0,
        span_col_end: 0,
    }),
})]
#[case(TestCase {
    id: TestId::IntLiteral,
    input: "a::b, reason=99999999999999999999999999999",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected string literal",
        span_text: Some("99999999999999999999999999999"),
        span_col_start: 13,
        span_col_end: 42,
    }),
})]
#[case(TestCase {
    id: TestId::AbsolutePathErr,
    input: "::a::b, reason=\"42\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "expected identifier",
        span_text: Some(":"),
        span_col_start: 0,
        span_col_end: 1,
    }),
})]
#[case(
TestCase {
    id: TestId::UnderscoreGroup,
    input: "error, _a::b, reason=\"42\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "report id should not contain",
        span_text: Some("_a::b"),
        span_col_start: 7,
        span_col_end: 12,
    }),
})]
#[case(
TestCase {
    id: TestId::UnderscoreReportId,
    input: "error, a::_b, reason=\"42\"",
    expected: ExpectedResult::Err(ErrExpected {
        message_contains: "report id should not contain",
        span_text: Some("a::_b"),
        span_col_start: 7,
        span_col_end: 12,
    }),
})]
fn test_parse_attrs(#[case] case: TestCase) {
    let tokens = syn::parse_str::<TokenStream>(case.input).unwrap();
    let result: Result<Arguments, syn::Error> = parse_mark_attrs(tokens);

    let case_id = case.id;

    /*
    // snippet to debug errors as assertion debug won't express them well.
    if let Err(error) = &result {
         println!(
            "{case_id:?} ERROR: {:?} {:?}: {:?}",
            error.span().source_text(),
            error.span().start(),
             error.span().end()
         );
    }
    */
    match &case.expected {
        ExpectedResult::Ok(ok) => {
            #[cfg(feature = "with-details")]
            {
                assert_matches!(
                    result,
                    Ok(Arguments { severity, report_ids, see_also, reason })
                    if severity == ok.severity &&
                    assert_paths(&report_ids, &ok.report_ids) &&
                    see_also == ok.see_also &&
                    reason == ok.reason,
                    "{case_id}"
                );
            }
            #[cfg(not(feature = "with-details"))]
            {
                let _ignored = ok;
                assert_matches!(result, Ok(Arguments {}), "{case_id}");
            }
        }
        ExpectedResult::Err(err) => {
            // Assume all tests are one-liners, otherwise expected structure need to be
            // adjusted.

            let expected_start = LineColumn {
                line: 1,
                column: err.span_col_start,
            };
            let expected_end = LineColumn {
                line: 1,
                column: err.span_col_end,
            };

            let expected_span_text =
                err.span_text.map(alloc::borrow::ToOwned::to_owned);
            assert_matches!(
                result, Err(error)
                if error.to_string().contains(err.message_contains) &&
                error.span().source_text() == expected_span_text &&
                error.span().start() == expected_start &&
                error.span().end() == expected_end,
                "{case_id}"
            );
        }
    }
}

#[cfg(feature = "with-details")]
fn assert_paths(actual: &[ReportId], expected: &[(&str, &str, usize, usize)]) -> bool {
    let converted: Vec<_> = actual
        .iter()
        .map(|path| {
            (
                path.group.as_str(),
                path.report_id.as_str(),
                path.span.start().column,
                path.span.end().column,
            )
        })
        .collect();

    assert_eq!(converted, expected);

    true
}

impl core::fmt::Display for TestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}: ")?;

        f.write_str(match self {
            Self::FullValidError => {
                "Valid input with Error severity, two paths, and reason"
            }
            Self::FullValidWarning => {
                "Valid input with Warning severity, two paths, and reason"
            }
            Self::FullValidInfo => {
                "Valid input with Info severity, two paths, and reason"
            }
            Self::FullValidDebug => {
                "Valid input with Debug severity, two paths, and empty reason"
            }
            Self::FullValidTrace => {
                "Valid input with Trace severity, two paths, and reason"
            }
            Self::NoSeverityValid => {
                "Valid input without severity, two paths, and reason"
            }
            Self::SinglePathValid => {
                "Valid input without severity, single path, and reason"
            }
            Self::MultiplePaths => {
                "Valid input without severity, three paths, and reason"
            }
            Self::SeeAlsoBeforeReason => {
                "Valid input with multiple see_also before reason"
            }
            Self::SeeAlsoAfterReason => {
                "Valid input with multiple see_also after reason"
            }
            Self::SeeAlsoMixedReason => {
                "Valid input with multiple see_also mixed with reason"
            }
            Self::MissingReason => "Missing reason after paths and severity",
            Self::OrderPathsBeforeReason => {
                "Missing paths before reason with severity present"
            }
            Self::OrderSeverityAfterPaths => "Severity appears after paths",
            Self::OrderSeeAlsoBeforePaths => "`See also` appears before paths",
            Self::DuplicateReason => "Reason appears twice",
            Self::ElementAfterReason => "Severity appears after reason",
            Self::UnknownSeverityValue => "Unknown severity value",
            Self::WrongKey => "Unknown key instead of reason",
            Self::PathThreeSegments => "Path with three segments",
            Self::ReasonWithoutPaths => "Reason without any paths",
            Self::DuplicateSeverity => "Severity specified twice",
            Self::SeverityOnly => "Only severity specified, missing paths and reason",
            Self::EmptyInput => "Empty input",
            Self::IntLiteral => "Integer literal instead of string for reason",
            Self::AbsolutePathErr => "Absolute path used, which is not allowed",
            Self::UnderscoreGroup => "Report group starts with `_`",
            Self::UnderscoreReportId => "Report Id starts with `_`",
        })
    }
}
