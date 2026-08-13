mod ident {
    #[scholium::mark(update_later::rust_1_96, reason = "Use core::assert_matches")]
    use assert_matches::assert_matches;
    use core::str::FromStr as _;

    use rstest::rstest;

    use crate::files::{Ident, IdentError};

    #[rstest]
    #[case("x", "x")]
    #[case("ident", "ident")]
    #[case("x_", "x_")]
    #[case("x_1234", "x_1234")]
    #[case("ß_1234", "ß_1234")]
    #[case("r#true", "true")]
    #[case("r#false", "false")]
    fn valid(#[case] input: &str, #[case] expected: &str) {
        assert_matches!(Ident::from_str(input), Ok(ident) if &*ident == expected);
    }

    #[rstest]
    #[case(("", IdentError::Empty))]
    #[case(underscore("_",))]
    #[case(underscore("_x1234"))]
    #[case(underscore("_ß1234"))]
    #[case(unicode("1234"))]
    #[case(unicode("x t"))]
    #[case(unicode("x "))]
    #[case(unicode(" x"))]
    fn error(#[case] (input, expected): (&str, IdentError)) {
        assert_eq!(Ident::from_str(input), Err(expected));
    }

    fn underscore(ident: &str) -> (&str, IdentError) {
        (ident, IdentError::Underscore(ident.to_owned()))
    }

    fn unicode(ident: &str) -> (&str, IdentError) {
        (ident, IdentError::Unicode(ident.to_owned()))
    }
}

mod report_id {
    use assert_matches::assert_matches;
    use core::str::FromStr as _;

    use rstest::rstest;

    use crate::files::{Context, IdentError, Prefix, ReportId, ReportIdError};

    #[rstest]
    #[case("group::y", "group::y")]
    #[case("false::true", "false::true")]
    #[case("r#false::r#true", "false::true")]
    #[case("group_123::id_456", "group_123::id_456")]
    #[case("ß::łąka", "ß::łąka")]
    fn valid(#[case] input: &str, #[case] expected: &str) {
        assert_matches!(ReportId::from_str(input), Ok(report_id) if report_id.to_string() == expected)
    }

    #[rstest]
    #[case("group", "id", "group::id")]
    #[case("false", "true", "false::true")]
    #[case("r#false", "r#true", "false::true")]
    #[case("group_123", "id_456", "group_123::id_456")]
    #[case("ß", "łąka", "ß::łąka")]
    fn valid_from_parts(#[case] group: &str, #[case] id: &str, #[case] expected: &str) {
        assert_matches!(ReportId::from_parts_raw(group, id), Ok(report_id) if report_id.to_string() == expected)
    }

    #[rstest]
    #[case(empty("", Context::Group))]
    #[case(empty("::", Context::Group))]
    #[case(empty("::id", Context::Group))]
    #[case(empty("group", Context::Id))]
    #[case(empty("group::", Context::Id))]
    #[case(unicode("1group", "1group", Context::Group))]
    #[case(unicode("1group::id", "1group", Context::Group))]
    #[case(unicode("1group::_id", "1group", Context::Group))]
    #[case(unicode("group::1id", "1id", Context::Id))]
    #[case(underscore("_group", "_group", Context::Group))]
    #[case(underscore("_group::id", "_group", Context::Group))]
    #[case(underscore("group::_id", "_id", Context::Id))]
    fn error(#[case] (input, expected): (&str, ReportIdError)) {
        assert_eq!(ReportId::from_str(input), Err(expected));
    }

    #[rstest]
    #[case(empty_p("", "", Context::Group))]
    #[case(empty_p("", "id", Context::Group))]
    #[case(empty_p("g", "", Context::Id))]
    #[case(unicode_p("1group", "", "1group", Context::Group))]
    #[case(unicode_p("1group", "_id", "1group", Context::Group))]
    #[case(unicode_p("1group", "id", "1group", Context::Group))]
    #[case(unicode_p("group", "1id", "1id", Context::Id))]
    #[case(underscore_p("_group", "", "_group", Context::Group))]
    #[case(underscore_p("_group", "id", "_group", Context::Group))]
    #[case(underscore_p("_group", "_id", "_group", Context::Group))]
    #[case(underscore_p("group", "_id", "_id", Context::Id))]
    fn error_from_parts(#[case] (group, id, expected): (&str, &str, ReportIdError)) {
        assert_eq!(ReportId::from_parts_raw(group, id), Err(expected));
    }

    #[rstest]
    #[case("group::id", "group", true)]
    #[case("group::id", "group::", true)]
    #[case("group::id", "group::i", true)]
    #[case("group::id", "group::id", true)]
    #[case("group::id", "group::o", false)]
    #[case("group::id", "group::other", false)]
    #[case("group::id", "other", false)]
    #[case("group::id", "other::", false)]
    #[case("group::id", "other::i", false)]
    #[case("group::id", "other::id", false)]
    fn starts_with_works(
        #[case] report_id: &str,
        #[case] prefix: &str,
        #[case] expected: bool,
    ) {
        let rid = ReportId::from_str(report_id).unwrap();
        let pfx = Prefix::from_str(prefix).unwrap();
        assert_eq!(rid.starts_with(&pfx), expected);
    }

    fn empty_p<'a>(
        group: &'a str,
        id: &'a str,
        context: Context,
    ) -> (&'a str, &'a str, ReportIdError) {
        (group, id, ReportIdError::Empty(context))
    }

    fn unicode_p<'a>(
        group: &'a str,
        id: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, &'a str, ReportIdError) {
        (
            group,
            id,
            ReportIdError::Ident(context, IdentError::Unicode(segment.to_owned())),
        )
    }

    fn underscore_p<'a>(
        group: &'a str,
        id: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, &'a str, ReportIdError) {
        (
            group,
            id,
            ReportIdError::Ident(context, IdentError::Underscore(segment.to_owned())),
        )
    }

    fn empty<'a>(input: &str, context: Context) -> (&str, ReportIdError) {
        (input, ReportIdError::Empty(context))
    }

    fn unicode<'a>(
        input: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, ReportIdError) {
        (
            input,
            ReportIdError::Ident(context, IdentError::Unicode(segment.to_owned())),
        )
    }

    fn underscore<'a>(
        input: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, ReportIdError) {
        (
            input,
            ReportIdError::Ident(context, IdentError::Underscore(segment.to_owned())),
        )
    }
}

mod prefix {
    use assert_matches::assert_matches;
    use core::str::FromStr as _;

    use rstest::rstest;

    use crate::files::{Context, IdentError, Prefix, PrefixError};

    #[rstest]
    #[case("x", "x")]
    #[case("x::y", "x::y")]
    #[case("x_123::y_456", "x_123::y_456")]
    #[case("false::true", "false::true")]
    #[case("r#false::r#true", "false::true")]
    #[case("ß::łąka", "ß::łąka")]
    #[case("group::", "group")]
    fn valid(#[case] input: &str, #[case] expected: &str) {
        assert_matches!(Prefix::from_str(input), Ok(prefix) if prefix.to_string() == expected)
    }

    #[rstest]
    #[case("x", None, "x")]
    #[case("x", Some(""), "x")]
    #[case("x", Some("y"), "x::y")]
    #[case("false", Some("true"), "false::true")]
    #[case("r#false", Some("r#true"), "false::true")]
    #[case("x_123", Some("y_456"), "x_123::y_456")]
    #[case("ß", Some("łąka"), "ß::łąka")]
    fn valid_from_parts(
        #[case] group: &str,
        #[case] id: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_matches!(Prefix::from_parts_raw(group, id), Ok(prefix) if prefix.to_string() == expected)
    }

    #[rstest]
    #[case(("", PrefixError::Empty))]
    #[case(("::", PrefixError::Empty))]
    #[case(("::id", PrefixError::Empty))]
    #[case(underscore("_group", "_group", Context::Group))]
    #[case(underscore("_group::id", "_group", Context::Group))]
    #[case(underscore("_group::_id", "_group", Context::Group))]
    #[case(unicode("1group", "1group", Context::Group))]
    #[case(unicode("1group::id", "1group", Context::Group))]
    #[case(unicode("1group::_id", "1group", Context::Group))]
    #[case(underscore("group::_id", "_id", Context::Id))]
    #[case(unicode("group::1id", "1id", Context::Id))]
    fn error(#[case] (input, expected): (&str, PrefixError)) {
        assert_eq!(Prefix::from_str(input), Err(expected));
    }

    #[rstest]
    #[case(("", None, PrefixError::Empty))]
    #[case(("", Some(""), PrefixError::Empty))]
    #[case(("", Some("id"), PrefixError::Empty))]
    #[case(underscore_p("_group", None, "_group", Context::Group))]
    #[case(underscore_p("_group", Some("id"), "_group", Context::Group))]
    #[case(underscore_p("group", Some("_id"), "_id", Context::Id))]
    #[case(unicode_p("1group", None, "1group", Context::Group))]
    #[case(unicode_p("1group", Some("id"), "1group", Context::Group))]
    #[case(unicode_p("1group", Some("_id"), "1group", Context::Group))]
    #[case(unicode_p("group", Some("1id"), "1id", Context::Id))]
    fn error_from_parts(
        #[case] (group, id, expected): (&str, Option<&str>, PrefixError),
    ) {
        assert_eq!(Prefix::from_parts_raw(group, id), Err(expected));
    }

    fn unicode_p<'a>(
        group: &'a str,
        id: Option<&'a str>,
        segment: &str,
        context: Context,
    ) -> (&'a str, Option<&'a str>, PrefixError) {
        (
            group,
            id,
            PrefixError::Ident(context, IdentError::Unicode(segment.to_owned())),
        )
    }

    fn underscore_p<'a>(
        group: &'a str,
        id: Option<&'a str>,
        segment: &str,
        context: Context,
    ) -> (&'a str, Option<&'a str>, PrefixError) {
        (
            group,
            id,
            PrefixError::Ident(context, IdentError::Underscore(segment.to_owned())),
        )
    }

    fn unicode<'a>(
        input: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, PrefixError) {
        (
            input,
            PrefixError::Ident(context, IdentError::Unicode(segment.to_owned())),
        )
    }

    fn underscore<'a>(
        input: &'a str,
        segment: &str,
        context: Context,
    ) -> (&'a str, PrefixError) {
        (
            input,
            PrefixError::Ident(context, IdentError::Underscore(segment.to_owned())),
        )
    }
}
