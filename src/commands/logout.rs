use clap::Parser;

use crate::config;
use crate::config::settings::{read_config_from, write_config_to};

#[derive(Parser)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let config_path = config::config_path();

    let mut cfg = match read_config_from(&config_path) {
        Ok(c) => c,
        Err(_) => {
            println!("You are not logged in.");
            return Ok(());
        }
    };

    if cfg.api_key.is_none() {
        println!("You are not logged in.");
        return Ok(());
    }

    cfg.api_key = None;
    cfg.name = None;

    write_config_to(&config_path, &cfg)?;

    println!("Successfully logged out. Have a great day!");

    Ok(())
}
