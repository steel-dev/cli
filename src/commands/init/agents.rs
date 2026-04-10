//! Detect installed coding agents and install the Steel skill into each.
//!
//! Currently supports Claude Code (`~/.claude/`) and Cursor (`~/.cursor/`).
//! The Steel skill content is embedded from `skills/steel-browser/SKILL.md`
//! so a single source of truth ships with the CLI binary.

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use dialoguer::Confirm;

use crate::status;

const SKILL_CONTENT: &str = include_str!("../../../skills/steel-browser/SKILL.md");

pub struct DetectedAgent {
    pub name: &'static str,
    pub install_path: PathBuf,
    pub content: String,
    pub already_up_to_date: bool,
}

pub fn detect_agents() -> Vec<DetectedAgent> {
    let mut agents = Vec::new();

    let Some(home) = dirs::home_dir() else {
        return agents;
    };

    // Claude Code — skills live at ~/.claude/skills/<name>/SKILL.md
    let claude_dir = home.join(".claude");
    if claude_dir.is_dir() {
        let install_path = claude_dir
            .join("skills")
            .join("steel-browser")
            .join("SKILL.md");
        let content = SKILL_CONTENT.to_string();
        let already_up_to_date = fs::read_to_string(&install_path)
            .map(|existing| existing == content)
            .unwrap_or(false);
        agents.push(DetectedAgent {
            name: "Claude Code",
            install_path,
            content,
            already_up_to_date,
        });
    }

    // Cursor — user-level rules at ~/.cursor/rules/*.mdc
    let cursor_dir = home.join(".cursor");
    if cursor_dir.is_dir() {
        let install_path = cursor_dir.join("rules").join("steel-browser.mdc");
        let content = cursor_rule_content();
        let already_up_to_date = fs::read_to_string(&install_path)
            .map(|existing| existing == content)
            .unwrap_or(false);
        agents.push(DetectedAgent {
            name: "Cursor",
            install_path,
            content,
            already_up_to_date,
        });
    }

    agents
}

/// Produce a Cursor `.mdc` rule file from the Claude Code SKILL.md by
/// stripping its YAML frontmatter and prepending Cursor-specific frontmatter.
fn cursor_rule_content() -> String {
    let body = strip_frontmatter(SKILL_CONTENT);
    format!(
        "---\ndescription: Steel browser automation CLI for web tasks (scrape, screenshot, interactive browser sessions)\nalwaysApply: false\n---\n\n{body}"
    )
}

fn strip_frontmatter(s: &str) -> &str {
    let Some(rest) = s.strip_prefix("---\n") else {
        return s;
    };
    let Some(end_idx) = rest.find("\n---\n") else {
        return s;
    };
    &rest[end_idx + "\n---\n".len()..]
}

pub fn install_skills_interactive() -> anyhow::Result<()> {
    let agents = detect_agents();

    if agents.is_empty() {
        status!(
            "No coding agents detected (looked for ~/.claude and ~/.cursor). Skipping skill install."
        );
        return Ok(());
    }

    status!("Detected coding agents:");
    for agent in &agents {
        let suffix = if agent.already_up_to_date {
            "  (already up-to-date)"
        } else {
            ""
        };
        status!(
            "  • {}  →  {}{}",
            agent.name,
            agent.install_path.display(),
            suffix
        );
    }

    if agents.iter().all(|a| a.already_up_to_date) {
        status!("All detected agents already have the current Steel skill installed.");
        return Ok(());
    }

    // Skip the interactive prompt when stdout is not a TTY (piped, sandboxed,
    // scripted invocations). The user can re-run `steel init` from a terminal
    // later to pick up skills.
    if !crate::util::output::is_tty() {
        status!(
            "Non-interactive session — skipping skill install. Re-run `steel init` from a terminal to install."
        );
        return Ok(());
    }

    let proceed = Confirm::new()
        .with_prompt("Install the Steel skill into these agents?")
        .default(true)
        .interact()?;

    if !proceed {
        status!("Skipped.");
        return Ok(());
    }

    for agent in &agents {
        if agent.already_up_to_date {
            continue;
        }
        install_one(agent)?;
        status!("  ✓ Installed Steel skill into {}", agent.name);
    }

    Ok(())
}

fn install_one(agent: &DetectedAgent) -> anyhow::Result<()> {
    if let Some(parent) = agent.install_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating directory {}", parent.display()))?;
    }
    fs::write(&agent.install_path, &agent.content)
        .with_context(|| format!("writing {}", agent.install_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_frontmatter_removes_yaml_header() {
        let input = "---\nname: test\n---\n\n# Body\ncontent";
        assert_eq!(strip_frontmatter(input), "\n# Body\ncontent");
    }

    #[test]
    fn strip_frontmatter_preserves_unfronted_text() {
        let input = "# No frontmatter\njust body";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn cursor_rule_content_has_cursor_frontmatter() {
        let rule = cursor_rule_content();
        assert!(rule.starts_with("---\ndescription:"));
        assert!(rule.contains("alwaysApply: false"));
        // The body (stripped of SKILL.md's original frontmatter) should still be present.
        assert!(rule.contains("# Steel"));
    }
}
