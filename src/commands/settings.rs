use clap::Parser;
use dialoguer::Select;

#[derive(Parser)]
pub struct Args;

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let config = crate::config::settings::read_config().ok();
    let current_instance = config
        .as_ref()
        .and_then(|c| c.instance.as_deref())
        .unwrap_or("cloud");

    let items = vec![
        format!(
            "Cloud{}",
            if current_instance == "cloud" {
                " (current)"
            } else {
                ""
            }
        ),
        format!(
            "Local{}",
            if current_instance == "local" {
                " (current)"
            } else {
                ""
            }
        ),
    ];

    let selection = Select::new()
        .with_prompt("Select instance type")
        .items(&items)
        .default(if current_instance == "local" { 1 } else { 0 })
        .interact()?;

    let value = if selection == 0 { "cloud" } else { "local" };

    let mut config = crate::config::settings::read_config().unwrap_or_default();
    config.instance = Some(value.to_string());
    crate::config::settings::write_config(&config)?;

    println!("Instance set to: {value}");

    Ok(())
}
