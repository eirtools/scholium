use proc_macro2::LineColumn;

use crate::files::ReportIdError;

#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Syn(String, LineColumn, syn::Error),
    ReportId(LineColumn, ReportIdError), // Should never happen
}
