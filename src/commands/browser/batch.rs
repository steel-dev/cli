//! Batch command: execute multiple browser actions in a single CLI invocation.
//!
//! Usage:
//!   steel browser batch "click @e3" "snapshot -i" --session my-task
//!   steel browser batch "fill @e1 Seoul" "fill @e2 Tokyo" "click @e5" --session my-task --bail

use anyhow::Result;
use clap::Parser;
use serde_json::{Value, json};

use crate::util::output;

use super::action::{self, ActionCommand};

#[derive(Parser)]
pub struct Args {
    /// Commands to execute (each as a quoted string, e.g. "click @e3")
    #[arg(required = true)]
    pub commands: Vec<String>,

    /// Stop on first error
    #[arg(long)]
    pub bail: bool,
}

/// Wrapper to parse a list of tokens as an ActionCommand via Clap.
#[derive(Parser)]
#[command(no_binary_name = true)]
struct ActionParser {
    #[command(subcommand)]
    action: ActionCommand,
}

/// Split a command string into tokens, respecting quoted substrings.
///
/// "fill @e5 \"hello world\"" → ["fill", "@e5", "hello world"]
fn split_command_str(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;

    for c in s.chars() {
        match (c, in_quote) {
            ('"', None) => in_quote = Some('"'),
            ('"', Some('"')) => in_quote = None,
            ('\'', None) => in_quote = Some('\''),
            ('\'', Some('\'')) => in_quote = None,
            (' ', None) if !current.is_empty() => {
                args.push(std::mem::take(&mut current));
            }
            (' ', None) => {}
            (c, _) => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

pub async fn run(args: Args, session: Option<&str>) -> Result<()> {
    let mut client = action::connect_daemon(session).await?;

    let mut results: Vec<Value> = Vec::new();
    let mut had_error = false;

    for cmd_str in &args.commands {
        let tokens = split_command_str(cmd_str);
        if tokens.is_empty() {
            continue;
        }

        // Parse tokens as an ActionCommand
        let parsed = match ActionParser::try_parse_from(&tokens) {
            Ok(p) => p,
            Err(e) => {
                let msg = e.render().to_string();
                if !output::is_json() {
                    eprintln!("Error ({}): {}", cmd_str, msg);
                }
                results.push(json!({
                    "command": cmd_str,
                    "success": false,
                    "error": msg,
                }));
                had_error = true;
                if args.bail {
                    break;
                }
                continue;
            }
        };

        // Convert to wire command
        let daemon_cmd = match parsed.action.into_daemon_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                let msg = e.to_string();
                if !output::is_json() {
                    eprintln!("Error ({}): {}", cmd_str, msg);
                }
                results.push(json!({
                    "command": cmd_str,
                    "success": false,
                    "error": msg,
                }));
                had_error = true;
                if args.bail {
                    break;
                }
                continue;
            }
        };

        // Execute
        match client.send(daemon_cmd).await {
            Ok(data) => {
                if !output::is_json() {
                    print_text_result(&data);
                }
                results.push(json!({
                    "command": cmd_str,
                    "success": true,
                    "data": data,
                }));
            }
            Err(e) => {
                let msg = e.to_string();
                if !output::is_json() {
                    eprintln!("Error ({}): {}", cmd_str, msg);
                }
                results.push(json!({
                    "command": cmd_str,
                    "success": false,
                    "error": msg,
                }));
                had_error = true;
                if args.bail {
                    break;
                }
            }
        }
    }

    drop(client);

    if output::is_json() {
        let envelope = json!({
            "success": !had_error,
            "data": { "results": results },
        });
        println!("{envelope}");
    }

    if had_error {
        if let Some(enriched) =
            action::session_health_check(session, &anyhow::anyhow!("batch error")).await
        {
            return Err(enriched);
        }
        anyhow::bail!("One or more batch commands failed");
    }

    Ok(())
}

/// Print a single result value in text mode.
fn print_text_result(data: &Value) {
    match data {
        // Snapshots come back as strings — print directly
        Value::String(s) => println!("{s}"),
        // Null means no meaningful output (e.g. press key)
        Value::Null => {}
        // Objects/arrays — compact JSON on one line
        other => {
            if let Ok(s) = serde_json::to_string(other) {
                println!("{s}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_simple() {
        assert_eq!(split_command_str("click @e3"), vec!["click", "@e3"]);
    }

    #[test]
    fn split_with_flag() {
        assert_eq!(split_command_str("snapshot -i"), vec!["snapshot", "-i"]);
    }

    #[test]
    fn split_quoted_value() {
        assert_eq!(
            split_command_str(r#"fill @e5 "hello world""#),
            vec!["fill", "@e5", "hello world"]
        );
    }

    #[test]
    fn split_single_quoted() {
        assert_eq!(
            split_command_str("fill @e5 'hello world'"),
            vec!["fill", "@e5", "hello world"]
        );
    }

    #[test]
    fn split_multiple_spaces() {
        assert_eq!(split_command_str("  click   @e3  "), vec!["click", "@e3"]);
    }

    #[test]
    fn split_empty() {
        assert!(split_command_str("").is_empty());
        assert!(split_command_str("   ").is_empty());
    }

    #[test]
    fn split_no_quotes_multi_word() {
        assert_eq!(
            split_command_str("fill @e5 Seoul"),
            vec!["fill", "@e5", "Seoul"]
        );
    }

    #[test]
    fn print_text_string() {
        // Just ensure it doesn't panic — actual stdout testing is integration-level
        print_text_result(&Value::String("hello".to_string()));
    }

    #[test]
    fn print_text_null() {
        print_text_result(&Value::Null);
    }

    #[test]
    fn print_text_object() {
        print_text_result(&json!({"clicked": "@e3"}));
    }
}
