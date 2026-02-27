# @steel-dev/cli

The CLI for Steel.dev.

## Install

Requires Node 18+
To run the Typescript examples, ensure that you have `ts-node` installed globally.

```bash
$ npm i @steel-dev/cli -g
```

## CLI

The full reference documentation of this CLI can be found in [cli-reference.md](docs/cli-reference.md).

Additional hand-maintained docs:

- [Browser compatibility](docs/browser-compat.md)
- [agent-browser migration guide](docs/migration-agent-browser.md)
- [References index](docs/references/README.md)
- [Synced browser commands reference](docs/references/steel-browser-commands.md)
- [Upstream sync guide](docs/upstream-sync.md)

```
USAGE
  $ steel [command] [options]

COMMANDS

⚡ Quickstart Commands

  forge               Start a new project using the Steel CLI
  run                 Run a Steel Cookbook automation instantly from the CLI — no setup, no files.


⏺︎ Other Commands

  browser
     └─ start          Create or attach a Steel browser session
     └─ stop           Stop the active Steel browser session
     └─ sessions       List browser sessions as JSON
     └─ live           Print current session live-view URL
  cache               Manage Steel CLI cache which is used to store files for quickly running scripts
  config              Display information about the current session
  dev
     └─ install        Install local Steel Browser runtime assets
     └─ start          Start local Steel Browser runtime containers
     └─ stop           Stop local Steel Browser runtime containers
  docs                Navigates to Steel Docs
  login               Login to Steel CLI
  logout              Logout from Steel CLI
  settings            Display current CLI settings (cloud/local)
  star                Opens the Steel Browser Repository in your browser
  support             Opens up the Steel Discord Server
  update              Update Steel CLI to the latest version

COMMON OPTIONS
  -h, --help          Display help for a command
  -v, --version       Display Steel CLI version
```

## Steel Browser Modes

`steel browser` is agent-browser compatible, with Steel sessions as the default bootstrap path.

- Cloud mode (default): no mode flag required.
- Self-hosted mode: use `--local`, or provide an explicit endpoint with `--api-url <url>`.

## Browser Session Quickstart

### Cloud (default)

```bash
steel login
steel browser start --session my-job
steel browser open https://example.com --session my-job
steel browser snapshot -i --session my-job
steel browser stop
```

### Self-hosted endpoint (Docker, GCP, Railway, etc.)

```bash
steel browser start --api-url https://steel.your-domain.dev/v1 --session my-job
steel browser open https://example.com --api-url https://steel.your-domain.dev/v1 --session my-job
```

## Endpoint Resolution Contract (Users and Agents)

For browser lifecycle and passthrough commands, endpoint selection should be deterministic.

Self-hosted endpoint precedence (highest to lowest):

1. `--api-url <url>`
2. `STEEL_BROWSER_API_URL` (canonical env var)
3. `STEEL_LOCAL_API_URL` (backward-compatible alias)
4. persisted setting (`browser.apiUrl` in `~/.config/steel/config.json`)
5. default `http://localhost:3000/v1`

Cloud endpoint precedence:

1. `STEEL_API_URL`
2. default `https://api.steel.dev/v1`

Attach-flag override rule:

- If `--cdp` or `--auto-connect` is provided, bootstrap injection is skipped and args are passed through unchanged.

## Local Runtime Flow (localhost self-hosting)

```bash
steel dev install
steel dev start
steel browser start --local --session local-job
steel browser open https://example.com --session local-job
steel browser stop
steel dev stop
```

Command intent:

- `steel dev install`: install/bootstrap local runtime assets only.
- `steel dev start`: start Docker runtime only.
- `steel dev stop`: stop Docker runtime only.
- `steel browser ... --local`: consume local runtime API/session lifecycle.

If runtime assets are missing, local browser flows return guidance to run `steel dev install`.
If runtime is installed but unavailable, local browser flows return guidance to run `steel dev start`.

## Agent Configuration Guidelines

- Always pass an explicit session name (`--session <name>`) for long-running agent jobs.
- In CI, set `STEEL_API_KEY` explicitly.
- For self-hosted agents, set one stable endpoint env var at process start and avoid mixing cloud/self-hosted within one process invocation.
- Prefer explicit `--api-url` when orchestrator behavior must be deterministic per command.

## Auto-Update

The Steel CLI automatically checks for updates when you run any command (except `help` and `update`). If a new version is available, you'll see a notification.

### Update Commands

```bash
# Update to the latest version
$ steel update

# Check for updates without installing
$ steel update --check

# Force update even if already on latest version
$ steel update --force
```

### Disabling Auto-Update Checks

You can disable automatic update checks in several ways:

```bash
# Using command line flag
$ steel run --no-update-check

# Using environment variable
$ STEEL_CLI_SKIP_UPDATE_CHECK=true steel run

# Auto-disabled in CI/test environments
$ CI=true steel run
$ NODE_ENV=test steel run
```

Update checks are cached for 24 hours to avoid unnecessary network requests.
