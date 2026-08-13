//! Tracing multiline console format an error bound to a source file and line.
//!
//! Fields supported:
//! * `file`: Filename, where error occurred.
//! * `line`: Line in file, where error occurred.
//! * `column`: Column in file, where error occurred.
//! * `message`: Message what exactly happened.
//! * `source_line`: Source line in filename if available when
//! * `error`: Rust error (if available).
use nu_ansi_term::Color;
use tracing_subscriber::fmt::FormatFields;

use super::FieldDetail;
use super::colors::Styled;

/// Formatter with detail given.
#[derive(Debug)]
pub(super) struct SourceErrorFormatter(FieldDetail);

impl SourceErrorFormatter {
    pub fn new(detail: FieldDetail) -> Self {
        Self(detail)
    }
}

const PREFIX_FILENAME_ARROW: &str = "  --> ";

impl<'writer> FormatFields<'writer> for SourceErrorFormatter {
    fn format_fields<R: tracing_subscriber::field::RecordFields>(
        &self,
        mut writer: tracing_subscriber::fmt::format::Writer<'writer>,
        fields: R,
    ) -> std::fmt::Result {
        let mut visitor = FieldVisitor::default();
        fields.record(&mut visitor);

        let (Some(file), Some(line), Some(column), Some(message)) =
            (visitor.file, visitor.line, visitor.column, visitor.message)
        else {
            return Ok(());
        };

        let line = line as usize;
        let column = column as usize + 1;
        let source_line = visitor.source_line;

        match (self.0, source_line) {
            // if source line is unknown, there's no need to express report in full
            // notation.
            (FieldDetail::Full, Some(source_line)) => {
                let is_ansi = writer.has_ansi_escapes();

                let caret_prefix = if column == 0 || column == 1 {
                    String::new()
                } else {
                    " ".repeat(column - 1)
                };

                let line_number = line.to_string();
                let width = line_number.len();
                let line_prefix = format!(" {line_number} | ");
                let line_prefix = Styled::prefix(is_ansi, &line_prefix);
                let white_prefix = format!(" {:>width$} | ", "");
                let white_prefix = Styled::prefix(is_ansi, &white_prefix);

                write!(writer, "{message}")?;
                if let Some(error) = visitor.error {
                    writeln!(writer, ": {error}")?;
                } else {
                    writeln!(writer)?;
                }

                writeln!(
                    writer,
                    "{}{file}:{line}:{column}",
                    Styled::prefix(is_ansi, PREFIX_FILENAME_ARROW)
                )?;

                writeln!(writer, "{white_prefix}")?;
                writeln!(writer, "{line_prefix}{source_line}")?;
                writeln!(
                    writer,
                    "{white_prefix}{caret_prefix}{}",
                    Styled::new(is_ansi, "^", Color::LightYellow.bold())
                )?;
                write!(writer, "{white_prefix}")?;
            }
            // expanded for clarity, could be collapsed as (_, _).
            (FieldDetail::Full, None) | (FieldDetail::Compact, _) => {
                write!(writer, "{file}:{line}:{column}: {message}")?;

                if let Some(error) = visitor.error {
                    write!(writer, ": {error}")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct FieldVisitor {
    file: Option<String>,
    line: Option<u64>,
    column: Option<u64>,
    source_line: Option<String>,
    report_id: Option<String>,
    error: Option<String>,
    message: Option<String>,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        match field.name() {
            "file" => self.file = Some(format!("{value:?}")),
            "source_line" => self.source_line = Some(format!("{value:?}")),
            "error" => self.error = Some(format!("{value:?}")),
            "message" => self.message = Some(format!("{value:?}")),
            "report_id" => self.report_id = Some(format!("{value:?}")),
            name => unreachable!(
                "Field name {name} (dbg) for error formatter is not supported"
            ),
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "line" => self.line = Some(value),
            "column" => self.column = Some(value),
            name => unreachable!(
                "Field name {name} (u64) for error formatter is not supported"
            ),
        }
    }
}
