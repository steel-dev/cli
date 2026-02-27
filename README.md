# @steel-dev/cli

Steel CLI for browser automation, API tools, and agent workflows.

This package now integrates `agent-browser` directly into `steel browser`, so you can keep familiar browser commands while using Steel-native session lifecycle, auth, and endpoint handling.

## Jump To

- [Install](#install)
- [Quick Start](#quick-start)
- [Agent-Browser Integration](#agent-browser-integration)
- [Command Overview](#command-overview)
- [Endpoint Resolution](#endpoint-resolution)
- [Documentation Map](#documentation-map)

## Install

Requirements:

- Node.js `>=18`

Install globally:

```bash
npm i -g @steel-dev/cli
```

Or run without installing globally:

```bash
npx @steel-dev/cli --help
```

## Quick Start

### Cloud mode (default)

```bash
steel login
steel browser start --session my-job
steel browser open https://example.com --session my-job
steel browser snapshot -i --session my-job
steel browser stop
```

### Self-hosted endpoint

```bash
steel browser start --api-url https://steel.your-domain.dev/v1 --session my-job
steel browser open https://example.com --api-url https://steel.your-domain.dev/v1 --session my-job
```

### Local runtime (`localhost` flow)

```bash
steel dev install
steel dev start
steel browser start --local --session local-job
steel browser open https://example.com --session local-job
steel browser stop
steel dev stop
```

## Agent-Browser Integration

`steel browser` is directly backed by the vendored `agent-browser` runtime.

Steel-owned lifecycle commands:

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`

All other `steel browser <command>` calls are inherited from upstream runtime behavior and routed through Steel.

Migration from upstream `agent-browser` is typically command-prefix only:

- before: `agent-browser <command> ...`
- after: `steel browser <command> ...`

Read more:

- [Migration guide](docs/migration-agent-browser.md)
- [Compatibility matrix](docs/browser-compat.md)
- [Steel browser reference](docs/references/steel-browser.md)
- [Synced command catalog](docs/references/steel-browser-commands.md)

## Command Overview

| Group               | Commands                                                                              |
| ------------------- | ------------------------------------------------------------------------------------- |
| Quickstart          | `forge`, `run`                                                                        |
| Browser lifecycle   | `browser start`, `browser stop`, `browser sessions`, `browser live`                   |
| Browser passthrough | `steel browser <inherited-command>`                                                   |
| API tools           | `scrape`, `screenshot`, `pdf`                                                         |
| Local runtime       | `dev install`, `dev start`, `dev stop`                                                |
| Account and utility | `login`, `logout`, `config`, `settings`, `cache`, `docs`, `support`, `star`, `update` |

For full flags and argument schemas, use the generated reference:

- [CLI reference](docs/cli-reference.md)

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

- `steel scrape` defaults to markdown-first output for token efficiency; use `--raw` for full JSON payload.
- `steel browser start` and `steel browser sessions` emit display-safe connect URLs with sensitive query values redacted.
- Browser command paths bypass auto-update checks for lower interactive latency.

## Auto-Update

```bash
# Update to latest
steel update

# Check without installing
steel update --check

# Force update
steel update --force
```

Disable automatic update checks:

```bash
steel run --no-update-check
STEEL_CLI_SKIP_UPDATE_CHECK=true steel run
CI=true steel run
NODE_ENV=test steel run
```

Update checks are cached for 24 hours.

## Documentation Map

Primary docs:

- [Docs index](docs/README.md)
- [Generated CLI reference](docs/cli-reference.md)
- [Browser compatibility](docs/browser-compat.md)
- [agent-browser migration guide](docs/migration-agent-browser.md)
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
