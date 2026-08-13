extern crate alloc;

mod cli;
mod command;
mod files;
mod tracing;

#[scholium::mark(
    trace,
    third_party::tracing_unstable,
    reason = "`valuable` integration is unstable",
    see_also = "https://github.com/tokio-rs/tracing/issues/1570"
)]
fn main() {
    command::process(cli::parse_args());
}
