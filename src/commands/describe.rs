use crate::util::output;
use anyhow::{Result, bail};
use clap::{ArgAction, CommandFactory, Parser};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Describe commands and parameters (structured introspection for AI agents)")]
pub struct Args {
    /// Command path to describe (e.g. "browser click")
    pub path: Vec<String>,
    /// Dump the entire command tree (recursive)
    #[arg(long)]
    pub all: bool,
}

// ── Output types ────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DescribeOutput {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcommands: Option<Vec<SubcommandInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterInfo>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_args: Vec<ParameterInfo>,
}

#[derive(Serialize)]
pub struct SubcommandInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub has_subcommands: bool,
    // Only present in --all mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcommands: Option<Vec<SubcommandInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterInfo>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_args: Vec<ParameterInfo>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub positional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
}

// ── Entry point ─────────────────────────────────────────────────────

pub async fn run(args: Args) -> Result<()> {
    let root = super::Cli::command();
    let (cmd, resolved_path) = resolve_command(&root, &args.path)?;

    let output_data = if args.all {
        describe_tree(&cmd, &resolved_path)
    } else {
        describe_node(&cmd, &resolved_path)
    };

    let value = serde_json::to_value(output_data)?;
    output::success_data(value);
    Ok(())
}

// ── Command resolution ──────────────────────────────────────────────

fn resolve_command<'a>(
    root: &'a clap::Command,
    path: &[String],
) -> Result<(clap::Command, Vec<String>)> {
    let mut current = root.clone();
    let mut resolved_path = vec![root.get_name().to_string()];

    for segment in path {
        let segment_lower = segment.to_lowercase();
        let found = current
            .get_subcommands()
            .find(|sub| {
                sub.get_name() == segment_lower || sub.get_all_aliases().any(|a| a == segment_lower)
            })
            .cloned();

        match found {
            Some(sub) => {
                resolved_path.push(sub.get_name().to_string());
                current = sub;
            }
            None => {
                let available: Vec<_> = current
                    .get_subcommands()
                    .filter(|s| !s.is_hide_set())
                    .filter(|s| s.get_name() != "help")
                    .map(|s| s.get_name().to_string())
                    .collect();
                bail!(
                    "Unknown command '{}' under '{}'. Available: {}",
                    segment,
                    resolved_path.join(" "),
                    available.join(", ")
                );
            }
        }
    }

    Ok((current, resolved_path))
}

// ── Describe single node ────────────────────────────────────────────

fn describe_node(cmd: &clap::Command, path: &[String]) -> DescribeOutput {
    let command_path = path.join(" ");
    let description = cmd.get_about().map(|s| s.to_string());
    let aliases = get_aliases(cmd);

    let visible_subs = get_visible_subcommands(cmd);
    let has_subs = !visible_subs.is_empty();

    if has_subs {
        let subcommands: Vec<SubcommandInfo> = visible_subs
            .iter()
            .map(|sub| SubcommandInfo {
                name: sub.get_name().to_string(),
                description: sub.get_about().map(|s| s.to_string()),
                aliases: get_aliases(sub),
                has_subcommands: has_visible_subcommands(sub),
                subcommands: None,
                parameters: None,
                global_args: Vec::new(),
            })
            .collect();

        DescribeOutput {
            command: command_path,
            description,
            aliases,
            subcommands: Some(subcommands),
            parameters: None,
            global_args: extract_global_args(cmd),
        }
    } else {
        DescribeOutput {
            command: command_path,
            description,
            aliases,
            subcommands: None,
            parameters: Some(extract_local_args(cmd)),
            global_args: extract_global_args(cmd),
        }
    }
}

// ── Describe full tree (--all) ──────────────────────────────────────

fn describe_tree(cmd: &clap::Command, path: &[String]) -> DescribeOutput {
    let command_path = path.join(" ");
    let description = cmd.get_about().map(|s| s.to_string());
    let aliases = get_aliases(cmd);

    let visible_subs = get_visible_subcommands(cmd);
    let has_subs = !visible_subs.is_empty();

    if has_subs {
        let subcommands: Vec<SubcommandInfo> = visible_subs
            .iter()
            .map(|sub| describe_subtree(sub, path))
            .collect();

        DescribeOutput {
            command: command_path,
            description,
            aliases,
            subcommands: Some(subcommands),
            parameters: None,
            global_args: extract_global_args(cmd),
        }
    } else {
        DescribeOutput {
            command: command_path,
            description,
            aliases,
            subcommands: None,
            parameters: Some(extract_local_args(cmd)),
            global_args: extract_global_args(cmd),
        }
    }
}

fn describe_subtree(cmd: &clap::Command, parent_path: &[String]) -> SubcommandInfo {
    let visible_subs = get_visible_subcommands(cmd);
    let has_subs = !visible_subs.is_empty();

    let mut child_path = parent_path.to_vec();
    child_path.push(cmd.get_name().to_string());

    if has_subs {
        let subcommands: Vec<SubcommandInfo> = visible_subs
            .iter()
            .map(|sub| describe_subtree(sub, &child_path))
            .collect();

        SubcommandInfo {
            name: cmd.get_name().to_string(),
            description: cmd.get_about().map(|s| s.to_string()),
            aliases: get_aliases(cmd),
            has_subcommands: true,
            subcommands: Some(subcommands),
            parameters: None,
            global_args: extract_global_args(cmd),
        }
    } else {
        SubcommandInfo {
            name: cmd.get_name().to_string(),
            description: cmd.get_about().map(|s| s.to_string()),
            aliases: get_aliases(cmd),
            has_subcommands: false,
            subcommands: None,
            parameters: Some(extract_local_args(cmd)),
            global_args: extract_global_args(cmd),
        }
    }
}

// ── Parameter extraction ────────────────────────────────────────────

const FILTERED_ARG_IDS: &[&str] = &["help", "version"];

fn is_visible_arg(arg: &clap::Arg) -> bool {
    !arg.is_hide_set() && !FILTERED_ARG_IDS.contains(&arg.get_id().as_str())
}

fn extract_local_args(cmd: &clap::Command) -> Vec<ParameterInfo> {
    cmd.get_arguments()
        .filter(|a| is_visible_arg(a) && !a.is_global_set())
        .map(arg_to_param)
        .collect()
}

fn extract_global_args(cmd: &clap::Command) -> Vec<ParameterInfo> {
    cmd.get_arguments()
        .filter(|a| is_visible_arg(a) && a.is_global_set())
        .map(arg_to_param)
        .collect()
}

fn arg_to_param(arg: &clap::Arg) -> ParameterInfo {
    let positional = arg.get_long().is_none() && arg.get_short().is_none();
    let param_type = infer_type(arg);
    let required = arg.is_required_set();
    let name = if let Some(long) = arg.get_long() {
        long.to_string()
    } else {
        arg.get_id().to_string()
    };

    let short = arg.get_short().map(|c| c.to_string());
    let default = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    let enum_values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .filter(|v| !v.is_hide_set())
        .map(|v| v.get_name().to_string())
        .collect();

    ParameterInfo {
        name,
        param_type,
        required,
        positional,
        description: arg.get_help().map(|s| s.to_string()),
        short,
        default,
        enum_values,
    }
}

fn infer_type(arg: &clap::Arg) -> String {
    match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse => "boolean".into(),
        ArgAction::Count => "integer".into(),
        ArgAction::Append => "string[]".into(),
        ArgAction::Set => {
            let num_args = arg.get_num_args().unwrap_or_else(|| (1..=1).into());
            if num_args.max_values() > 1 {
                return "string[]".into();
            }
            let possible = arg.get_possible_values();
            if !possible.is_empty() {
                "enum".into()
            } else {
                "string".into()
            }
        }
        _ => "string".into(),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn get_aliases(cmd: &clap::Command) -> Vec<String> {
    cmd.get_all_aliases().map(|a| a.to_string()).collect()
}

fn get_visible_subcommands(cmd: &clap::Command) -> Vec<clap::Command> {
    cmd.get_subcommands()
        .filter(|s| !s.is_hide_set() && s.get_name() != "help")
        .cloned()
        .collect()
}

fn has_visible_subcommands(cmd: &clap::Command) -> bool {
    cmd.get_subcommands()
        .any(|s| !s.is_hide_set() && s.get_name() != "help")
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn root_cmd() -> clap::Command {
        super::super::Cli::command()
    }

    fn describe_path(path: &[&str]) -> Value {
        let root = root_cmd();
        let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        let (cmd, resolved) = resolve_command(&root, &path_owned).unwrap();
        let output = describe_node(&cmd, &resolved);
        serde_json::to_value(output).unwrap()
    }

    fn describe_path_all(path: &[&str]) -> Value {
        let root = root_cmd();
        let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        let (cmd, resolved) = resolve_command(&root, &path_owned).unwrap();
        let output = describe_tree(&cmd, &resolved);
        serde_json::to_value(output).unwrap()
    }

    #[test]
    fn root_lists_all_visible_commands() {
        let v = describe_path(&[]);
        let subs = v["subcommands"].as_array().unwrap();
        let names: Vec<&str> = subs.iter().map(|s| s["name"].as_str().unwrap()).collect();

        // Visible commands should be present
        assert!(names.contains(&"browser"));
        assert!(names.contains(&"scrape"));
        assert!(names.contains(&"login"));

        // Hidden __daemon should not appear
        assert!(!names.contains(&"__daemon"));
    }

    #[test]
    fn browser_includes_flattened_actions() {
        let v = describe_path(&["browser"]);
        let subs = v["subcommands"].as_array().unwrap();
        let names: Vec<&str> = subs.iter().map(|s| s["name"].as_str().unwrap()).collect();

        // Flattened actions should appear as direct subcommands
        assert!(names.contains(&"navigate"));
        assert!(names.contains(&"click"));
        assert!(names.contains(&"snapshot"));
        assert!(names.contains(&"start"));
        assert!(names.contains(&"stop"));
    }

    #[test]
    fn click_parameters() {
        let v = describe_path(&["browser", "click"]);
        let params = v["parameters"].as_array().unwrap();

        let selector = params.iter().find(|p| p["name"] == "selector").unwrap();
        assert_eq!(selector["type"], "string");
        assert_eq!(selector["required"], true);
        assert_eq!(selector["positional"], true);

        let button = params.iter().find(|p| p["name"] == "button").unwrap();
        assert_eq!(button["type"], "string");
        assert_eq!(button["required"], false);
        assert_eq!(button["positional"], false);

        let new_tab = params.iter().find(|p| p["name"] == "new-tab").unwrap();
        assert_eq!(new_tab["type"], "boolean");
        assert_eq!(new_tab["required"], false);
    }

    #[test]
    fn navigate_aliases() {
        let v = describe_path(&["browser", "navigate"]);
        let aliases = v["aliases"].as_array().unwrap();
        let alias_strs: Vec<&str> = aliases.iter().map(|a| a.as_str().unwrap()).collect();
        assert!(alias_strs.contains(&"open"));
        assert!(alias_strs.contains(&"goto"));
    }

    #[test]
    fn resolve_by_alias() {
        let v = describe_path(&["browser", "open"]);
        // Should resolve to navigate
        assert!(v["command"].as_str().unwrap().ends_with("navigate"));
    }

    #[test]
    fn unknown_command_errors() {
        let root = root_cmd();
        let path = vec!["nonexistent".to_string()];
        let result = resolve_command(&root, &path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown command"));
    }

    #[test]
    fn all_flag_dumps_full_tree() {
        let v = describe_path_all(&["browser"]);
        let subs = v["subcommands"].as_array().unwrap();

        // In --all mode, subcommands with children should have nested subcommands
        let get = subs.iter().find(|s| s["name"] == "get").unwrap();
        assert_eq!(get["has_subcommands"], true);
        assert!(get["subcommands"].is_array());
        let get_subs = get["subcommands"].as_array().unwrap();
        let get_names: Vec<&str> = get_subs
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert!(get_names.contains(&"text"));
        assert!(get_names.contains(&"url"));

        // Leaf commands should have parameters
        let click = subs.iter().find(|s| s["name"] == "click").unwrap();
        assert!(click["parameters"].is_array());
    }

    #[test]
    fn default_value_included() {
        let v = describe_path(&["browser", "wait"]);
        let params = v["parameters"].as_array().unwrap();
        let timeout = params.iter().find(|p| p["name"] == "timeout").unwrap();
        assert_eq!(timeout["default"], "30000");
    }

    #[test]
    fn vec_args_typed_as_array() {
        let v = describe_path(&["browser", "fill"]);
        let params = v["parameters"].as_array().unwrap();
        let value = params.iter().find(|p| p["name"] == "value").unwrap();
        assert_eq!(value["type"], "string[]");
    }

    #[test]
    fn global_args_separated() {
        // At the browser level, session is global and should appear in global_args
        let v = describe_path(&["browser"]);
        let global = v["global_args"].as_array().unwrap();
        let global_names: Vec<&str> = global.iter().map(|g| g["name"].as_str().unwrap()).collect();
        assert!(global_names.contains(&"session"));

        // At the root level, json and local are global
        let root = describe_path(&[]);
        let root_global = root["global_args"].as_array().unwrap();
        let root_global_names: Vec<&str> = root_global
            .iter()
            .map(|g| g["name"].as_str().unwrap())
            .collect();
        assert!(root_global_names.contains(&"json"));
        assert!(root_global_names.contains(&"local"));

        // Global args should not appear in subcommand list
        let subs = root["subcommands"].as_array().unwrap();
        let sub_names: Vec<&str> = subs.iter().map(|s| s["name"].as_str().unwrap()).collect();
        assert!(!sub_names.contains(&"json"));
    }

    #[test]
    fn hidden_args_excluded() {
        let v = describe_path(&[]);
        let global = v["global_args"].as_array().unwrap();
        let global_names: Vec<&str> = global.iter().map(|g| g["name"].as_str().unwrap()).collect();

        // no-update-check is hidden
        assert!(!global_names.contains(&"no-update-check"));
    }
}
