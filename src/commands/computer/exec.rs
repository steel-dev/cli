use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

use super::resolve_computer_id;
use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::status;
use crate::util::api;
use crate::util::output::{self, SilentExit};

pub const TIMEOUT_EXIT_CODE: i32 = 124;
const MAX_WAKE_ATTEMPTS: u32 = 8;
const MAX_WAKE_DELAY: Duration = Duration::from_secs(10);

#[derive(Parser)]
pub struct Args {
    /// Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
    pub computer_id: Option<String>,

    /// Shell command string, run by /bin/sh -c
    #[arg(short = 'c', long = "command", conflicts_with = "argv")]
    pub command: Option<String>,

    /// Working directory inside the computer
    #[arg(long)]
    pub cwd: Option<String>,

    /// Environment variable for the command, repeatable
    #[arg(long = "env", value_name = "KEY=VALUE")]
    pub env: Vec<String>,

    /// Kill the command after this many seconds (at most 3600)
    #[arg(long = "timeout", value_name = "SECONDS")]
    pub timeout_seconds: Option<u32>,

    /// Program and arguments, given after `--`
    #[arg(last = true, value_name = "ARGV")]
    pub argv: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum Event {
    Start,
    Output {
        data: String,
    },
    Keepalive,
    Exit {
        #[serde(rename = "exitCode")]
        exit_code: i32,
        #[serde(rename = "timedOut")]
        timed_out: bool,
    },
}

#[derive(Debug, Deserialize)]
pub struct ExecResult {
    pub output: String,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
    #[serde(rename = "timedOut")]
    pub timed_out: bool,
    #[serde(default)]
    pub truncated: bool,
}

pub fn build_body(args: &Args, stream: bool) -> Result<Value> {
    let mut body = match (&args.command, args.argv.is_empty()) {
        (Some(command), true) => json!({ "command": command }),
        (None, false) => json!({ "argv": args.argv }),
        (None, true) => bail!("Give a command after `--` or with -c."),
        (Some(_), false) => bail!("Give either a command after `--` or -c, not both."),
    };
    if let Some(cwd) = &args.cwd {
        body["cwd"] = json!(cwd);
    }
    if !args.env.is_empty() {
        body["env"] = json!(parse_env(&args.env)?);
    }
    if let Some(timeout) = args.timeout_seconds {
        if timeout == 0 || timeout > 3600 {
            bail!("--timeout must be between 1 and 3600 seconds.");
        }
        body["timeoutSeconds"] = json!(timeout);
    }
    body["stream"] = json!(stream);
    Ok(body)
}

pub fn parse_env(entries: &[String]) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for entry in entries {
        let Some((key, value)) = entry.split_once('=') else {
            bail!("--env expects KEY=VALUE, got {entry:?}.");
        };
        if key.is_empty() {
            bail!("--env expects KEY=VALUE, got {entry:?}.");
        }
        env.insert(key.to_string(), value.to_string());
    }
    Ok(env)
}

pub fn parse_event(line: &[u8]) -> Result<Option<Event>> {
    let text = std::str::from_utf8(line).context("the computer sent a non-UTF-8 event")?;
    if text.trim().is_empty() {
        return Ok(None);
    }
    let event = serde_json::from_str(text).context("the computer sent a malformed event")?;
    Ok(Some(event))
}

pub const fn exit_code(code: i32, timed_out: bool) -> i32 {
    if timed_out {
        TIMEOUT_EXIT_CODE
    } else if code < 0 {
        1
    } else {
        code
    }
}

pub async fn run(args: Args) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let stream = !output::is_json();
    let body = build_body(&args, stream)?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let response = send(&client, &base_url, mode, &auth, &id, body).await?;
    let code = if stream {
        stream_output(response).await?
    } else {
        collect_output(response).await?
    };
    if code != 0 {
        return Err(SilentExit(code).into());
    }
    Ok(())
}

async fn send(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    id: &str,
    body: Value,
) -> Result<reqwest::Response> {
    let mut attempt = 0;
    loop {
        match client
            .exec_computer(base_url, mode, auth, id, body.clone())
            .await
        {
            Ok(response) => return Ok(response),
            Err(ApiError::RetryLater { seconds, .. }) if attempt < MAX_WAKE_ATTEMPTS => {
                attempt += 1;
                let delay = Duration::from_secs(seconds.max(1)).min(MAX_WAKE_DELAY);
                status!(
                    "Computer {id} is not ready yet, retrying in {}s.",
                    delay.as_secs()
                );
                tokio::time::sleep(delay).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

async fn stream_output(mut response: reqwest::Response) -> Result<i32> {
    let mut stdout = tokio::io::stdout();
    let mut buffer: Vec<u8> = Vec::new();
    let mut result: Option<i32> = None;
    while let Some(chunk) = response
        .chunk()
        .await
        .context("the connection to the computer dropped")?
    {
        buffer.extend_from_slice(&chunk);
        while let Some(end) = buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=end).collect();
            if let Some(event) = parse_event(&line[..line.len() - 1])? {
                handle_event(event, &mut stdout, &mut result).await?;
            }
        }
    }
    if let Some(event) = parse_event(&buffer)? {
        handle_event(event, &mut stdout, &mut result).await?;
    }
    match result {
        Some(code) => Ok(code),
        None => bail!("The connection closed before the command finished."),
    }
}

async fn handle_event(
    event: Event,
    stdout: &mut tokio::io::Stdout,
    result: &mut Option<i32>,
) -> Result<()> {
    match event {
        Event::Output { data } => {
            stdout.write_all(data.as_bytes()).await?;
            stdout.flush().await?;
        }
        Event::Exit {
            exit_code: code,
            timed_out,
        } => *result = Some(exit_code(code, timed_out)),
        Event::Start | Event::Keepalive => {}
    }
    Ok(())
}

async fn collect_output(response: reqwest::Response) -> Result<i32> {
    let text = response
        .text()
        .await
        .context("the connection to the computer dropped")?;
    let value: Value = serde_json::from_str(text.trim_start())
        .context("the computer returned a malformed result")?;
    let result: ExecResult = serde_json::from_value(value.clone())
        .context("the computer returned a malformed result")?;
    output::success_data(value);
    Ok(exit_code(result.exit_code, result.timed_out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(command: Option<&str>, argv: &[&str]) -> Args {
        Args {
            computer_id: None,
            command: command.map(str::to_string),
            cwd: None,
            env: vec![],
            timeout_seconds: None,
            argv: argv.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn body_takes_exactly_one_command_form() {
        assert_eq!(
            build_body(&args(Some("echo hi"), &[]), true).unwrap(),
            json!({ "command": "echo hi", "stream": true })
        );
        assert_eq!(
            build_body(&args(None, &["ls", "-la"]), false).unwrap(),
            json!({ "argv": ["ls", "-la"], "stream": false })
        );
        assert!(build_body(&args(None, &[]), true).is_err());
        assert!(build_body(&args(Some("id"), &["id"]), true).is_err());
    }

    #[test]
    fn body_carries_cwd_env_and_timeout() {
        let mut a = args(Some("id"), &[]);
        a.cwd = Some("/tmp".into());
        a.env = vec!["A=1".into(), "B=x=y".into()];
        a.timeout_seconds = Some(30);
        assert_eq!(
            build_body(&a, true).unwrap(),
            json!({
                "command": "id",
                "cwd": "/tmp",
                "env": { "A": "1", "B": "x=y" },
                "timeoutSeconds": 30,
                "stream": true
            })
        );
        a.timeout_seconds = Some(0);
        assert!(build_body(&a, true).is_err());
        a.timeout_seconds = Some(3601);
        assert!(build_body(&a, true).is_err());
    }

    #[test]
    fn env_entries_need_a_key() {
        assert!(parse_env(&["=1".into()]).is_err());
        assert!(parse_env(&["novalue".into()]).is_err());
        assert_eq!(parse_env(&["K=".into()]).unwrap()["K"], "");
    }

    #[test]
    fn events_parse_and_map_to_exit_codes() {
        assert!(matches!(
            parse_event(br#"{"event":"start"}"#).unwrap(),
            Some(Event::Start)
        ));
        assert!(matches!(
            parse_event(br#"{"event":"output","data":"hi\n"}"#).unwrap(),
            Some(Event::Output { data }) if data == "hi\n"
        ));
        assert!(matches!(
            parse_event(br#"{"event":"exit","exitCode":3,"timedOut":false}"#).unwrap(),
            Some(Event::Exit {
                exit_code: 3,
                timed_out: false
            })
        ));
        assert!(parse_event(b"   ").unwrap().is_none());
        assert!(parse_event(b"not json").is_err());

        assert_eq!(exit_code(0, false), 0);
        assert_eq!(exit_code(3, false), 3);
        assert_eq!(exit_code(-1, false), 1);
        assert_eq!(exit_code(0, true), TIMEOUT_EXIT_CODE);
    }

    #[test]
    fn collected_results_deserialize() {
        let result: ExecResult = serde_json::from_str(
            r#"{"output":"hi\n","exitCode":0,"timedOut":false,"truncated":false}"#,
        )
        .unwrap();
        assert_eq!(result.output, "hi\n");
        assert!(!result.truncated);
    }
}
