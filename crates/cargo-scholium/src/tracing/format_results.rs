//! Format reports for console output.
use tracing_subscriber::fmt::FormatFields;

use super::FieldDetail;
use super::colors::Styled;

#[derive(Debug)]
pub(super) struct ResultFormatter(FieldDetail);

impl ResultFormatter {
    pub fn new(detail: FieldDetail) -> Self {
        Self(detail)
    }
}

const PREFIX_FILENAME_ARROW: &str = "  --> ";
const PREFIX_SEE_ALSO: &str = "  = see also: ";
const PREFIX_REASON: &str = "  = reason: ";
const PREFIX_INFO: &str = "  = info: ";
const PREFIX_HELP: &str = "  = help: ";

impl<'writer> FormatFields<'writer> for ResultFormatter {
    fn format_fields<R: tracing_subscriber::field::RecordFields>(
        &self,
        mut writer: tracing_subscriber::fmt::format::Writer<'writer>,
        fields: R,
    ) -> std::fmt::Result {
        let mut visitor = FieldVisitor::default();
        fields.record(&mut visitor);

        let (Some(file), Some(line), Some(column), Some(reason), Some(message)) = (
            visitor.file,
            visitor.line,
            visitor.column,
            visitor.reason,
            visitor.message,
        ) else {
            return Ok(());
        };

        let line = line as usize;
        let column = column as usize + 1;

        match self.0 {
            FieldDetail::Full => {
                writeln!(writer, "{message}")?;
                let is_ansi = writer.has_ansi_escapes();

                writeln!(
                    writer,
                    "{}{file}:{line}:{column}",
                    Styled::prefix(is_ansi, PREFIX_FILENAME_ARROW)
                )?;

                write!(
                    writer,
                    "{}{reason} ",
                    Styled::prefix(is_ansi, PREFIX_REASON)
                )?;

                for see_also in visitor.see_also {
                    writeln!(writer)?;
                    write!(
                        writer,
                        "{}{see_also}",
                        Styled::prefix(is_ansi, PREFIX_SEE_ALSO)
                    )?;
                }

                if let Some(info) = visitor.info {
                    if !info.is_empty() {
                        writeln!(writer)?;
                        write!(
                            writer,
                            "{}{info}",
                            Styled::prefix(is_ansi, PREFIX_INFO)
                        )?;
                    }
                }

                if let Some(report_id) = visitor.report_id {
                    writeln!(writer)?;
                    write!(
                        writer,
                        "{}for further information run `cargo scholium explain \
                         {report_id}`",
                        Styled::prefix(is_ansi, PREFIX_HELP)
                    )?;
                }
            }
            FieldDetail::Compact => {
                write!(writer, "{file}:{line}:{column}: {message}: {reason}")?;
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
    reason: Option<String>,
    see_also: Vec<String>,
    report_id: Option<String>,
    info: Option<String>,
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
            "report_id" => self.report_id = Some(format!("{value:?}")),
            "info" => self.info = Some(format!("{value:?}")),
            "see_also" => {
                self.see_also = serde_json::from_str(&format!("{value:?}"))
                    .expect("Vec<String> is expected")
            }
            "reason" => self.reason = Some(format!("{value:?}")),
            "message" => self.message = Some(format!("{value:?}")),
            name => unreachable!(
                "Field name {name} (dbg) for result formatter is not supported"
            ),
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "line" => self.line = Some(value),
            "column" => self.column = Some(value),
            name => unreachable!(
                "Field name {name} (u64) for result formatter is not supported"
            ),
        }
    }
}
