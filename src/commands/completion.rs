use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use std::io;

#[derive(Parser)]
#[command(
    about = "Generate a shell completion script",
    long_about = "Generate a shell completion script for the given shell.\n\n\
                  Install examples:\n  \
                  # Bash (user-local)\n  \
                  steel completion bash > ~/.local/share/bash-completion/completions/steel\n\n  \
                  # Zsh (ensure a writable dir in $fpath)\n  \
                  steel completion zsh > \"${fpath[1]}/_steel\"\n\n  \
                  # Fish\n  \
                  steel completion fish > ~/.config/fish/completions/steel.fish\n\n  \
                  # PowerShell (profile)\n  \
                  steel completion powershell | Out-String | Invoke-Expression"
)]
pub struct Args {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

pub async fn run(args: Args) -> Result<()> {
    let mut cmd = super::Cli::command();
    let name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(shell: Shell) -> String {
        let mut cmd = super::super::Cli::command();
        let mut buf = Vec::new();
        generate(shell, &mut cmd, "steel", &mut buf);
        String::from_utf8(buf).expect("completion output is UTF-8")
    }

    #[test]
    fn bash_completion_references_binary() {
        let out = render(Shell::Bash);
        assert!(out.contains("steel"));
        assert!(!out.is_empty());
    }

    #[test]
    fn zsh_completion_has_compdef_header() {
        let out = render(Shell::Zsh);
        assert!(out.contains("#compdef steel"));
    }

    #[test]
    fn fish_completion_emits_complete_commands() {
        let out = render(Shell::Fish);
        assert!(out.contains("complete -c steel"));
    }
}
