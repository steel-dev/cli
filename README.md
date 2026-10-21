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

### Agent setup and first use

```bash
curl -fsS https://setup.steel.dev | sh -s -- --agent --from opencode
```

Use `claude-code`, `codex`, `cursor`, `opencode`, or `pi` as the target. Agent
setup installs `steel-browser` globally for that target without asking the user
to choose a skill. Other skills remain available through `steel skills install`.
Skill installation uses Node.js/npm and git; npm enforces the selected `skills`
package's current engine requirement. The native CLI itself needs neither Node
nor the `@steel-dev/cli` npm migration shim. Older npm releases contain the
legacy Node CLI; do not use either as a fallback for native CLI reachability.

An installer cannot change the parent agent's environment. Use the native path
in **each** subsequent tool command instead of repairing PATH manually:

```bash
"$HOME/.steel/bin/steel" --version
"$HOME/.steel/bin/steel" init --agent --from opencode
```

The target is saved, so interrupted setup can rerun `init --agent` without
`--from`. A failed stage returns nonzero with a next action. Pending device
authorization returns an explicit pending/incomplete result; approve the displayed
code in a browser on any machine, then rerun setup or `login --resume`.
No localhost callback or copying credentials into the conversation is needed.

#### Device authorization

Device authorization uses a browser handoff and saves a pending request, so it can
be resumed without a localhost callback:

```bash
"$HOME/.steel/bin/steel" login --no-wait --no-browser
# Approve the displayed code in a browser, then:
"$HOME/.steel/bin/steel" login --resume --no-browser
"$HOME/.steel/bin/steel" doctor --preflight
```

The authentication service configuration is intentionally explicit: set
`STEEL_AUTH_ISSUER`, `STEEL_AUTH_AUDIENCE`, and `STEEL_API_URL` before the first
device login using values supplied by your Steel deployment. The issuer, token
audience, and product API transport are separate settings; `--api-url` selects
self-hosted/local mode and is not a substitute for the product transport.
Existing API-key automation remains supported without device-login settings.

Settings and pending authorization survive subsequent commands in private
`auth.json` under `STEEL_CONFIG_DIR` (default `~/.config/steel`). Inconsistent
environment overrides fail closed rather than sending tokens to another API. The
client sends product access tokens using `Authorization: Bearer`; API keys still
use `Steel-Api-Key`. Browser CDP uses session connection credentials returned by
the product API, never a product access token disguised as an API key.

Secret state is written atomically with private Unix permissions. A cross-process
lock serializes polling, refresh rotation, and logout. If refresh is interrupted
after it may have reached the issuer, the client refuses to replay the rotating
token and asks for reauthorization. Normal `logout` revokes the refresh grant;
`logout --local-only` is explicit offline recovery and does not revoke the server
grant or already-issued access tokens.

Agent init returns the handoff immediately and checks a saved request when it is
due. Human login waits in bounded increments; interruption does not lose the
request or backoff. `--resume` never silently creates a replacement for an
expired or denied request. A token or `doctor --preflight` result proves neither
browser capacity nor CDP connectivity.

After installation, load `steel-browser` through the client's supported skill
mechanism. Setup reports loading and task completion as unverified, not ready.
As a separate task, ask:

> Use Steel to open https://books.toscrape.com/, find A Light in the Attic,
> open its detail page, and tell me its price and availability.

Omit an explicit session timeout to use the API's account-compatible default.
Verify the answer independently and stop/release the session. Session creation
and a successful CLI command do not prove task success.

The public installer uses the latest stable release. Source changes require a
release before the installer can provide them.

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
| Cloud computers     | `computer create`, `computer exec`, `computer ssh`, `computer list`, `computer use`  |
| Secrets             | `secret list`, `secret create`, `secret get`, `secret update`, `secret delete`      |
| Environments        | `environment list`, `environment create`, `environment update`, `environment delete` |
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

See the [compatibility matrix](docs/browser-compat.md), [Steel browser reference](docs/references/steel-browser.md), and [synced command catalog](docs/references/steel-browser-commands.md) for details.

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
- [Upstream sync guide](docs/upstream-sync.md)

Reference docs:

- [References index](docs/references/README.md)
- [Steel CLI reference (high-level)](docs/references/steel-cli.md)
- [Steel browser reference (modes + contracts)](docs/references/steel-browser.md)
- [Steel browser commands (synced + transformed)](docs/references/steel-browser-commands.md)
- [Agent troubleshooting playbook](docs/references/agent-troubleshooting.md)
- [Pinned upstream source snapshot](docs/references/upstream/agent-browser-commands.source.md)

Agent skills:

- [Steel Skills catalog](https://github.com/steel-dev/skills)
- [Steel Agent Skills docs](https://docs.steel.dev/overview/agent-skills)
- [agent-browser migration guide](https://github.com/steel-dev/skills/blob/main/steel-browser/references/migration-agent-browser.md)

---

Licensed under [MIT](./LICENSE).
