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

```
USAGE
  $ steel [command] [options]

COMMANDS

⚡ Quickstart Commands

  forge               Start a new project using the Steel CLI
  run                 Run a Steel Cookbook automation instantly from the CLI — no setup, no files.


⏺︎ Other Commands

  browser
     └─ start          Create or attach a Steel browser cloud session
     └─ stop           Stop the active Steel browser session
     └─ sessions       List browser sessions as JSON
     └─ live           Print current session live-view URL
  cache               Manage Steel CLI cache which is used to store files for quickly running scripts
  config              Display information about the current session
  dev
     └─ start          Start local Steel Browser runtime via Docker Compose
     └─ stop           Stop local Steel Browser runtime
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
