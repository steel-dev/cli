# Steel CLI

> Browser infrastructure for AI agents, from your terminal.

<!-- [![crates.io](https://img.shields.io/crates/v/steel-cli?color=orange&logo=rust)](https://crates.io/crates/steel-cli) -->

[![npm](https://img.shields.io/npm/v/@steel-dev/cli?color=cb3837&logo=npm&label=npm)](https://www.npmjs.com/package/@steel-dev/cli)
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
- `npm i -g @steel-dev/cli` (thin wrapper around the native binary)

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
| Browser passthrough | `steel browser <inherited-command>`                                                  |
| Browser profiles    | `profile import`, `profile sync`, `profile list`, `profile delete`                   |
| API tools           | `scrape`, `screenshot`, `pdf`                                                        |
| Local runtime       | `dev install`, `dev start`, `dev stop`                                               |
| Credentials         | `credentials list`, `credentials create`, `credentials update`, `credentials delete` |
| Account and utility | `login`, `logout`, `config`, `doctor`, `cache`, `update`, `completion`               |

Full flags and schemas: [CLI reference](docs/cli-reference.md).

## Agent-Browser Integration

`steel browser` is directly backed by the vendored [`agent-browser`](https://github.com/steel-dev/agent-browser) runtime. Steel-owned lifecycle commands:

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`

All other `steel browser <command>` calls inherit upstream runtime behavior and route through Steel. Migration from upstream `agent-browser` is typically command-prefix only:

- before: `agent-browser <command> ...`
- after: `steel browser <command> ...`

See the [migration guide](skills/steel-browser/references/migration-agent-browser.md), [compatibility matrix](docs/browser-compat.md), and [synced command catalog](docs/references/steel-browser-commands.md) for details.

## Endpoint Resolution

For browser lifecycle, browser passthrough bootstrap, and top-level API tools (`scrape`, `screenshot`, `pdf`), endpoint selection is deterministic.

Self-hosted precedence (highest to lowest):

1. `--api-url <url>`
2. `STEEL_BROWSER_API_URL`
3. `STEEL_LOCAL_API_URL`
4. `browser.apiUrl` in `~/.config/steel/config.json`
5. `http://localhost:3000/v1`

Cloud precedence:

1. `STEEL_API_URL`
2. `https://api.steel.dev/v1`

Attach-flag override rule:

- If `--cdp` or `--auto-connect` is provided, Steel bootstrap injection is skipped and passthrough args are forwarded unchanged.

## Output and Runtime Notes

- `steel scrape` defaults to markdown-first output for token efficiency; use `--raw` for the full JSON payload.
- `steel browser start` and `steel browser sessions` emit display-safe connect URLs with sensitive query values redacted.
- Browser command paths bypass auto-update checks for lower interactive latency.
- Piped output auto-switches to JSON for machine-readable workflows. Use `--json` to force it, or `STEEL_FORCE_TTY=1` to disable.

## Documentation Map

Primary docs:

- [Docs index](docs/README.md)
- [Generated CLI reference](docs/cli-reference.md)
- [Browser compatibility](docs/browser-compat.md)
- [agent-browser migration guide](skills/steel-browser/references/migration-agent-browser.md)
- [Upstream sync guide](docs/upstream-sync.md)

Reference docs:

- [References index](docs/references/README.md)
- [Steel CLI reference (high-level)](docs/references/steel-cli.md)
- [Steel browser reference (modes + contracts)](docs/references/steel-browser.md)
- [Steel browser commands (synced + transformed)](docs/references/steel-browser-commands.md)
- [Agent troubleshooting playbook](docs/references/agent-troubleshooting.md)
- [Pinned upstream source snapshot](docs/references/upstream/agent-browser-commands.source.md)

Agent skill:

- [Steel Browser skill package](skills/steel-browser/README.md)

## Migrating from the Node CLI

The Steel CLI is now a single native Rust binary. If you previously installed it via `npm i -g @steel-dev/cli`, the `install.sh` script auto-detects and removes the old Node-based install. You can also do it manually:

```bash
npm update -g @steel-dev/cli    # auto-installs native binary
npm uninstall -g @steel-dev/cli # optional cleanup
export PATH="$HOME/.steel/bin:$PATH"
```

---

Licensed under [MIT](./LICENSE).
