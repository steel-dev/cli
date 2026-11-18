mod api;
mod auto_update;
mod browser;
mod commands;
mod config;
mod telemetry;
mod util;

pub use auto_update::{is_child as is_auto_update_child, run_child as run_auto_update_child};
pub use commands::{Cli, parse, run};
pub use telemetry::{is_flush_child, run_flush_child};
pub use util::output::handle_error;
