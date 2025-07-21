# @steel-dev/cli

The CLI for Steel.dev

## Install

Requires Node 18+
To run the Typescript examples, ensure that you have `ts-node` installed globally.

```bash
$ npm i @steel-dev/cli -g
```

## CLI

```
$ steel --help

Usage: steel [options] [command]

Options:
  -v, --version                                       Show version number
  -h, --help                                          Show help

Commands:
  forge [directory to scaffold new Steel project]     Start a new project using the Steel CLI
  run [example Steel project to run]                  Run a Steel project
  update                                              Update Steel CLI to the latest version
  files                                               Files Endpoint
  login                                               Login to Steel CLI
  logout                                              Logout from Steel CLI
  sessions                                            Sessions Endpoint
  browser start                                       Starts the development environment
  browser stop                                        Stops the development server
  tools                                               Tools Endpoint
  help [command]                                      Show help for command
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
