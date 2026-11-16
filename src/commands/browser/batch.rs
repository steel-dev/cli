use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;

use super::action::{ActionCommand, Browser, Plan};

#[derive(Parser)]
pub struct Args {
    /// Commands to run in order, each as one string (e.g. "click e3" or "fill e3 'New York'"). They stop at the first failure.
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

pub fn split_command_str(s: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut started = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' if !started || current.ends_with('=') => {
                started = true;
                loop {
                    match chars.next() {
                        Some(q) if q == c && chars.peek().is_none_or(|n| n.is_whitespace()) => {
                            break;
                        }
                        Some('\\') if c == '"' && matches!(chars.peek(), Some('"' | '\\')) => {
                            current.extend(chars.next());
                        }
                        Some(other) => current.push(other),
                        None if c == '"' => return Err("unclosed double quote".to_string()),
                        None => return Err("unclosed single quote".to_string()),
                    }
                }
            }
            c if c.is_whitespace() => {
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            c => {
                started = true;
                current.push(c);
            }
        }
    }
    if started {
        args.push(current);
    }
    Ok(args)
}

pub async fn run(args: Args, session: Option<&str>, diff: bool) -> Result<()> {
    let mut steps = Vec::new();
    for line in &args.commands {
        let tokens = split_command_str(line).map_err(|err| anyhow!("{line}: {err}"))?;
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

    fn split(s: &str) -> Vec<String> {
        split_command_str(s).unwrap()
    }

    #[test]
    fn split_simple() {
        assert_eq!(split("click @e3"), vec!["click", "@e3"]);
    }

    #[test]
    fn split_with_flag() {
        assert_eq!(split("snapshot -i"), vec!["snapshot", "-i"]);
    }

    #[test]
    fn split_quoted_value() {
        assert_eq!(
            split(r#"fill @e5 "hello world""#),
            vec!["fill", "@e5", "hello world"]
        );
    }

    #[test]
    fn split_single_quoted() {
        assert_eq!(
            split("fill @e5 'hello world'"),
            vec!["fill", "@e5", "hello world"]
        );
    }

    #[test]
    fn split_multiple_spaces() {
        assert_eq!(split("  click   @e3  "), vec!["click", "@e3"]);
        assert_eq!(split("fill e3\tSeoul"), vec!["fill", "e3", "Seoul"]);
    }

    #[test]
    fn split_empty() {
        assert!(split("").is_empty());
        assert!(split("   ").is_empty());
    }

    #[test]
    fn split_no_quotes_multi_word() {
        assert_eq!(split("fill @e5 Seoul"), vec!["fill", "@e5", "Seoul"]);
    }

    #[test]
    fn apostrophes_inside_words_are_kept() {
        assert_eq!(
            split("fill e3 it's fine"),
            vec!["fill", "e3", "it's", "fine"]
        );
        assert_eq!(
            split("fill e3 'it's fine'"),
            vec!["fill", "e3", "it's fine"]
        );
        assert_eq!(
            split(r#"fill e3 "it's fine""#),
            vec!["fill", "e3", "it's fine"]
        );
    }

    #[test]
    fn quotes_inside_words_are_kept() {
        assert_eq!(
            split(r#"eval document.querySelector("h1").textContent"#),
            vec!["eval", r#"document.querySelector("h1").textContent"#]
        );
        assert_eq!(
            split(r#"eval "document.querySelector("h1").textContent""#),
            vec!["eval", r#"document.querySelector("h1").textContent"#]
        );
        assert_eq!(
            split("eval 'document.querySelector('h1').textContent'"),
            vec!["eval", "document.querySelector('h1').textContent"]
        );
    }

    #[test]
    fn empty_quotes_make_an_empty_argument() {
        assert_eq!(split(r#"fill e3 """#), vec!["fill", "e3", ""]);
        assert_eq!(split("fill e3 ''"), vec!["fill", "e3", ""]);
    }

    #[test]
    fn double_quotes_take_escapes_and_attach_to_flags() {
        assert_eq!(
            split(r#"fill e3 "say \"hi\" \\o/""#),
            vec!["fill", "e3", r#"say "hi" \o/"#]
        );
        assert_eq!(
            split(r#"find button --name="Save draft""#),
            vec!["find", "button", "--name=Save draft"]
        );
        assert_eq!(
            split("wait --text='Signed in'"),
            vec!["wait", "--text=Signed in"]
        );
        assert_eq!(split(r"fill e3 C:\temp"), vec!["fill", "e3", r"C:\temp"]);
    }

    #[test]
    fn unclosed_quotes_are_errors() {
        assert!(split_command_str(r#"fill e3 "abc"#).is_err());
        assert!(split_command_str("fill e3 'abc").is_err());
    }

    #[tokio::test]
    async fn rejects_commands_that_must_run_alone() {
        for line in [
            "tab switch t2",
            "tab new",
            "tab close",
            "status x-1",
            "cancel x-1",
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
                commands: vec!["dialog accept".to_string()],
                bail: false,
            },
            Some("unused"),
            false,
        )
        .await
        .unwrap_err();
        assert!(format!("{err:#}").contains("`steel browser dialog` was removed"));

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
