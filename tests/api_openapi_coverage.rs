//! Ensures generated API metadata points at real CLI commands.

use std::collections::BTreeSet;

use clap::CommandFactory;

use steel_cli::api::generated::CLI_OPERATION_METADATA;
use steel_cli::commands;

fn generated_operation_ids_are_unique() {
    let operation_ids: Vec<_> = CLI_OPERATION_METADATA
        .iter()
        .map(|operation| operation.id)
        .collect();
    let unique_ids: BTreeSet<_> = operation_ids.iter().copied().collect();
    assert_eq!(operation_ids.len(), unique_ids.len());
}

fn generated_command_paths() -> BTreeSet<String> {
    CLI_OPERATION_METADATA
        .iter()
        .map(|operation| {
            operation
                .command
                .split_whitespace()
                .filter(|part| !part.starts_with('-'))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

#[test]
fn generated_operation_ids_are_unique_test() {
    generated_operation_ids_are_unique();
}

#[test]
fn generated_command_paths_exist_in_clap_tree() {
    let root = commands::Cli::command();

    for command_path in generated_command_paths() {
        let mut current = root.clone();
        for segment in command_path.split_whitespace() {
            let next = current
                .get_subcommands()
                .find(|subcommand| subcommand.get_name() == segment)
                .unwrap_or_else(|| panic!("generated command path does not exist: {command_path}"))
                .clone();
            current = next;
        }
    }
}
