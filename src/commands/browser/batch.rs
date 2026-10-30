use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;

use super::action::{ActionCommand, Browser, Plan};

#[derive(Parser)]
pub struct Args {
    /// Commands to run in order (each as a quoted string, e.g. "click e3"). They stop at the first failure.
    #[arg(required = true)]
    pub commands: Vec<String>,

    #[arg(long, hide = true)]
    pub bail: bool,
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct ActionParser {
    #[command(subcommand)]
    action: ActionCommand,
}

const MAX_COMMANDS: usize = 50;

pub fn split_command_str(s: &str) -> Vec<String> {
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

pub async fn run(args: Args, session: Option<&str>, diff: bool) -> Result<()> {
    let mut steps = Vec::new();
    for line in &args.commands {
        let tokens = split_command_str(line);
        if tokens.is_empty() {
            continue;
        }
        let parsed = ActionParser::try_parse_from(&tokens)
            .map_err(|e| anyhow!("{line}: {}", e.render().to_string().trim()))?;
        match parsed.action.plan().with_context(|| line.clone())? {
            Plan::Run(step) if step.batchable() => steps.push(step),
            _ => bail!("`{line}` cannot run inside batch. Run it on its own."),
        }
    }
    if steps.is_empty() {
        bail!("No commands to run.");
    }
    if steps.iter().map(|step| step.commands.len()).sum::<usize>() > MAX_COMMANDS {
        bail!("A batch can run at most {MAX_COMMANDS} commands.");
    }

    let mut browser = Browser::open(session, true).await?;
    browser.execute(steps, diff).await
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

    #[tokio::test]
    async fn rejects_commands_that_must_run_alone() {
        for line in [
            "tab switch t2",
            "tab new",
            "tab close",
            "dialog accept",
            "close",
            "status x-1",
        ] {
            let err = run(
                Args {
                    commands: vec![line.to_string()],
                    bail: false,
                },
                Some("unused"),
                false,
            )
            .await
            .unwrap_err();
            assert!(
                err.to_string().contains("cannot run inside batch"),
                "{line}: {err}"
            );
        }
    }

    #[tokio::test]
    async fn rejects_removed_and_invalid_commands_before_starting_a_session() {
        let err = run(
            Args {
                commands: vec!["cookies".to_string()],
                bail: false,
            },
            Some("unused"),
            false,
        )
        .await
        .unwrap_err();
        assert!(format!("{err:#}").contains("was removed in 0.5"));

        let err = run(
            Args {
                commands: vec!["fly away".to_string()],
                bail: false,
            },
            Some("unused"),
            false,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().starts_with("fly away:"));
    }
}
