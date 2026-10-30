# Steel CLI

> Browser infrastructure for AI agents, from your terminal.

<!-- [![crates.io](https://img.shields.io/crates/v/steel-cli?color=orange&logo=rust)](https://crates.io/crates/steel-cli) -->

[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Discord](https://img.shields.io/badge/discord-join-5865F2?logo=discord&logoColor=white)](https://discord.gg/steel-dev)

The Steel CLI gives any AI agent (Claude, Cursor, Browser Use, or your own) a production-grade cloud browser: markdown-first scraping, long-running sessions, stealth, residential proxies, automatic CAPTCHA handling, and credential injection. One binary, cloud or self-hosted.

```bash
# Install, then interactively log in and install agent skills
curl -fsS https://setup.steel.dev | sh

# Scrape a page (agent-ready markdown)
steel scrape https://example.com
```

<p align="center">
  <img src="./assets/demo.gif" alt="Steel CLI demo">
</p>

## Why Steel CLI

- **Agent-native.** Markdown-first output, `steel describe` for machine-readable command introspection, shell completions, and ready-to-install skills for Claude Code, Cursor, OpenCode, and Codex.
- **Long-running sessions.** 24-hour browser sessions, reusable named profiles, persistent auth context. Built for agents that actually live: deep research, async workflows, overnight jobs.
- **Framework-agnostic.** First-class support for Claude Computer Use, OpenAI Computer Use, Gemini Computer Use, Browser Use, Playwright, Puppeteer, Selenium, Stagehand, CrewAI, Magnitude, Notte, Agno, and AgentKit.
- **Cloud or self-hosted.** Fully open source. Use managed infrastructure at [steel.dev](https://steel.dev), or run the [Steel Browser](https://github.com/steel-dev/steel-browser) stack yourself with `steel dev` or Docker.
- **Pro browser features.** Managed residential proxies, stealth mode, automatic CAPTCHA solving, stored credential injection. Everything a real-world agent needs.

## Links

- 📖 [Docs](https://docs.steel.dev): guides, recipes, API reference
- 🧪 [Cookbook](https://github.com/steel-dev/steel-cookbook): runnable examples across Python, Node, and every supported framework
- 💬 [Discord](https://discord.gg/steel-dev): community, feedback, roadmap
- 🐙 [GitHub](https://github.com/steel-dev/cli): source, issues, releases

## Install

```bash
curl -fsS https://setup.steel.dev | sh
```

The installer drops a single binary into `~/.steel/bin`, updates your shell's `PATH`, and auto-generates shell completions for bash, zsh, and fish.

Alternatives:

- Download from [GitHub Releases](https://github.com/steel-dev/cli/releases)
- `cargo install steel-cli`

After install, either restart your shell or run `export PATH="$HOME/.steel/bin:$PATH"`.

## Shell Completions

Shell completions are installed automatically by `install.sh`. To regenerate or install manually:

```bash
# Bash (user-local)
steel completion bash > ~/.local/share/bash-completion/completions/steel

# Zsh (ensure a writable dir is in $fpath, then reload)
steel completion zsh > "${fpath[1]}/_steel"

# Fish
steel completion fish > ~/.config/fish/completions/steel.fish

# PowerShell (append to $PROFILE)
steel completion powershell | Out-String | Invoke-Expression
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

## Command Overview

| Group               | Commands                                                                             |
| ------------------- | ------------------------------------------------------------------------------------ |
| Onboarding          | `init`, `forge`                                                                      |
| Browser lifecycle   | `browser start`, `browser stop`, `browser sessions`, `browser live`                  |
| Browser actions     | `browser open`, `browser snapshot`, `browser click`, `browser fill`, `browser batch`, … |
| Browser profiles    | `profile import`, `profile sync`, `profile list`, `profile delete`                   |
| API tools           | `scrape`, `screenshot`, `pdf`                                                        |
| Local runtime       | `dev install`, `dev start`, `dev stop`                                               |
| Credentials         | `credentials list`, `credentials create`, `credentials update`, `credentials delete` |
| Account and utility | `login`, `logout`, `config`, `doctor`, `cache`, `update`, `completion`               |

Full flags and schemas: [CLI reference](docs/cli-reference.md).

## How `steel browser` Works

Each `steel browser` action is one request to the session's commands API. The browser engine runs inside the Steel session next to Chrome, so the CLI keeps no process running between commands.

```bash
steel browser open https://example.com
steel browser snapshot -i
steel browser click e5 --diff
steel browser batch "fill e3 Seoul" "click e7" "snapshot -i"
```

- The `default` session starts on first use. Use `--session <name>` for more, or `--session <session-id>` to drive a session created elsewhere.
- Sessions are released after 10 minutes without commands (`steel browser start --inactivity-timeout <ms>`).
- A retried command never runs twice. Run the same command again after a lost response to get the stored result.

See the [Steel browser reference](docs/references/steel-browser.md) and the [command catalog](docs/references/steel-browser-commands.md) for details.

## Endpoint Resolution

Every command that calls the Steel API uses one URL, chosen in this order:

1. `--api-url <url>`
2. `--local`: `browser.apiUrl` in `~/.config/steel/config.json`, or `http://localhost:3000/v1`
3. `STEEL_API_URL`
4. `https://api.steel.dev/v1`

URLs on `steel.dev` are Steel Cloud and need an API key. Any other URL is treated as self-hosted.

## Output and Runtime Notes

- `steel scrape` prints markdown by default for token efficiency; use `--json` for the full API response.
- `steel browser start` and `steel browser sessions` emit display-safe connect URLs with sensitive query values redacted.
- Browser command paths bypass auto-update checks for lower interactive latency.
- Output is text by default, including when piped. Pass `--json` for machine-readable output.

## Documentation Map

Primary docs:

- [Docs index](docs/README.md)
- [Generated CLI reference](docs/cli-reference.md)

Reference docs:

- [References index](docs/references/README.md)
- [Steel CLI reference (high-level)](docs/references/steel-cli.md)
- [Steel browser reference (modes + contracts)](docs/references/steel-browser.md)
- [Steel browser commands](docs/references/steel-browser-commands.md)
- [Agent troubleshooting playbook](docs/references/agent-troubleshooting.md)

Agent skills:

- [Steel Skills catalog](https://github.com/steel-dev/skills)
- [Steel Agent Skills docs](https://docs.steel.dev/overview/agent-skills)

---

Licensed under [MIT](./LICENSE).
