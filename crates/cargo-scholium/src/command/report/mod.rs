mod definitions;
mod error;
mod fs_walker;
mod imp;
mod parse;
mod log;

use super::{CommandError, obtain_metadata};

use error::ProcessError;
pub use imp::process;
