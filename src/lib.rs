#![allow(clippy::too_many_arguments)]

mod api;
mod browser;
mod commands;
mod config;
mod telemetry;
mod util;

pub use commands::{Cli, parse, run};
pub use util::output::handle_error;
