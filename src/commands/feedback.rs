use clap::{Parser, ValueEnum};
use serde_json::{Value, json};

use crate::api::client::SteelClient;
use crate::util::output;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Category {
    /// Steel failed or did the wrong thing
    Bug,
    /// Steel's docs are wrong, missing something or unclear
    Docs,
    /// Steel lacks something you need
    Product,
    /// Anything else
    Other,
}

impl Category {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Docs => "docs",
            Self::Product => "product",
            Self::Other => "other",
        }
    }
}

#[derive(Parser)]
#[command(after_help = "Examples:
  steel feedback bug \"browser start fails every time\" --command \"steel browser start\" --error \"internal_error\"
  steel feedback docs \"The timeout default is wrong\" --page sessions/timeouts
  steel feedback product \"I need to download files from a session\"")]
pub struct Args {
    /// What kind of feedback this is
    #[arg(value_enum)]
    pub category: Category,

    /// What happened or what you need, in a few sentences
    pub message: String,

    /// What you were trying to do
    #[arg(long)]
    pub goal: Option<String>,

    /// The Steel command that failed, with its arguments
    #[arg(long)]
    pub command: Option<String>,

    /// The error or output you got
    #[arg(long)]
    pub error: Option<String>,

    /// For docs feedback, the docs page it is about
    #[arg(long)]
    pub page: Option<String>,
}

impl Args {
    fn body(&self) -> Value {
        let mut body = json!({
            "category": self.category.as_str(),
            "message": self.message,
        });
        for (key, value) in [
            ("goal", &self.goal),
            ("command", &self.command),
            ("error", &self.error),
            ("page", &self.page),
        ] {
            if let Some(value) = value {
                body[key] = json!(value);
            }
        }
        body
    }
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    output::suppress_feedback_hint();
    let response = SteelClient::new()?.send_feedback(&args.body()).await?;
    let id = response
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();

    output::success(
        json!({ "id": id }),
        &format!("Sent to the Steel team. Reference: {id}\n"),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(category: Category) -> Args {
        Args {
            category,
            message: "browser start fails".into(),
            goal: None,
            command: None,
            error: None,
            page: None,
        }
    }

    #[test]
    fn body_carries_only_the_fields_given() {
        assert_eq!(
            args(Category::Product).body(),
            json!({"category": "product", "message": "browser start fails"})
        );
        let full = Args {
            goal: Some("open a page".into()),
            command: Some("steel browser start".into()),
            error: Some("internal_error".into()),
            page: Some("sessions/timeouts".into()),
            ..args(Category::Bug)
        };
        assert_eq!(
            full.body(),
            json!({
                "category": "bug",
                "message": "browser start fails",
                "goal": "open a page",
                "command": "steel browser start",
                "error": "internal_error",
                "page": "sessions/timeouts",
            })
        );
    }
}
