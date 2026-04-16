# Changelog

All notable changes to this project will be documented in this file.

## [v0.3.8] - 2026-04-16

### Security

- Upgraded `rustls-webpki` to patch RUSTSEC-2026-0098 and RUSTSEC-2026-0099.

### Features

- Simplified the `--agent` onboarding flow: `steel init --agent` now runs the full init (detect agents, install the Steel skill, print next steps) with all prompts auto-accepted, instead of just printing the onboarding guide and exiting. The `install.sh` agent-mode path no longer requires a TTY and always prints the agent guide on first run.

### Changed

- `steel forge` now downloads templates from `https://registry.steel-edge.net/versions/<version>/<template>` instead of `https://raw.githubusercontent.com/steel-dev/steel-cookbook/...`. Scripts or mirrors that pinned to the raw GitHub URL need to update.

### Bug Fixes

- Fixed `steel forge` archive extraction so the top-level template directory is preserved (removed an incorrect `--strip-components=1` flag passed to `tar`).

### Miscellaneous

- Pinned `aes`, `cbc`, and `hmac` back to `0.8` / `0.1` / `0.12` after the Dependabot bumps to `0.9` / `0.2` / `0.13` pulled in an incompatible `digest` / `crypto-common` chain. ([#71](https://github.com/steel-dev/cli/pull/71), [#72](https://github.com/steel-dev/cli/pull/72), [#73](https://github.com/steel-dev/cli/pull/73))
- Bumped `tokio` to `1.51.1` and `zip` to `8.5.1`. ([#74](https://github.com/steel-dev/cli/pull/74), [#75](https://github.com/steel-dev/cli/pull/75))


## [v0.3.7] - 2026-04-13

### Features

- Added `steel init`, a one-command onboarding flow that chains `login`, a `doctor` preflight, and auto-installs the Steel skill into detected coding agents (Claude Code, Cursor, etc.). Run `steel init --agent` to print the agent onboarding guide instead.
- Added `steel browser batch`, which runs multiple browser commands (navigate, click, fill, snapshot, …) in a single invocation against one session.

### Bug Fixes

- Fixed cookie re-encryption to operate on raw bytes end-to-end instead of round-tripping through `String::from_utf8_lossy`, which could corrupt cookie values containing non-UTF-8 bytes.
- `steel doctor` now returns a proper error on failure instead of calling `process::exit` directly, so it composes cleanly when invoked from `steel init` and other callers.


## [v0.3.6] - 2026-04-06

### Features

- Added `steel doctor`, a diagnostic command that reports on the local CLI environment and surfaces common misconfigurations.
- Profile subcommands (`profile import`, `profile sync`, `profile delete`) now accept the profile name as a positional argument (e.g. `steel profile delete myprofile`). The old `--name` flag is kept as a hidden alias so existing scripts keep working.
- Added installation instructions to the `steel-browser` skill so agents can self-install the CLI when missing.

### Changed

- `steel browser start --json` now emits the full, unsanitized `connectUrl` so automation can use it directly. Scripts that expected the redacted form need to redact themselves if they log the payload.

### Bug Fixes

- Fixed several Linux cookie-decryption issues: added KWallet support, corrected keyring names for Opera and Vivaldi, treated unrecognized cookie prefixes as plaintext, and fell back to an empty password when the keyring lookup returns nothing.
- Specify the account when querying the macOS Keychain so the correct cookie key is returned.
- Corrected the YAML frontmatter in the `steel-browser` skill so it parses cleanly.
- Avoided double-scanning browser profiles during profile operations.

### Documentation

- Updated the `steel-browser` skill with refreshed `scrape` and `browser` command examples. ([#70](https://github.com/steel-dev/cli/pull/70))

### Miscellaneous

- Pinned `sha1` and `sha2` back to 0.10 after the 0.11 bumps landed via Dependabot but broke the transitive `digest` chain; added a dependabot ignore to prevent the regression.
- Routine dependency bumps: `tokio` 1.50 → 1.51, `zip` 8.3 → 8.5, `libc` 0.2.183 → 0.2.184, `hmac` 0.12.1 → 0.13.0, `proptest` 1.10 → 1.11. ([#61](https://github.com/steel-dev/cli/pull/61), [#63](https://github.com/steel-dev/cli/pull/63), [#64](https://github.com/steel-dev/cli/pull/64), [#66](https://github.com/steel-dev/cli/pull/66), [#67](https://github.com/steel-dev/cli/pull/67), [#68](https://github.com/steel-dev/cli/pull/68), [#69](https://github.com/steel-dev/cli/pull/69))
- Refactored `BrowserId` to implement `FromStr`, and added profile e2e tests covering real-profile cookie verification. ([#60](https://github.com/steel-dev/cli/issues/60))
- Removed the redirect flag from the install script and dropped some dead code.


## [v0.3.5] - 2026-03-25

### Security

- Patched `rustls-webpki` vulnerability and dropped the now-unneeded `RUSTSEC-2025-0119` ignore from `deny.toml`.

### Features

- Added `steel describe` command for inspecting resources.
- `steel` now auto-detects piped output and switches to JSON mode automatically, and commands return semantic exit codes for easier scripting (covers `cache`, `dev`, `forge`, `login`, `logout`, `profile`, `update`, and `browser start`).
- Browser action subcommands now auto-start a session if one isn't already running, so you no longer need a separate `browser start` before issuing actions.
- Updated the `steel-browser` SKILL.md and added an evals harness. ([#56](https://github.com/steel-dev/cli/issues/56))

### Changed

- `steel install` now carries an agent-facing docstring and skill setup instructions to guide onboarding. ([#55](https://github.com/steel-dev/cli/issues/55))

### Bug Fixes

- Restored browser action flag parity between `steel browser action` and `steel describe`.
- Cleaned up incorrect references in `bin/steel.js`, `scripts/postinstall.js`, and the `steel-browser` skill docs.

### Miscellaneous

- Dependency bumps: `getrandom` 0.3.4 → 0.4.2 ([#53](https://github.com/steel-dev/cli/pull/53)) and `zip` 8.2.0 → 8.3.0 ([#54](https://github.com/steel-dev/cli/pull/54)).
- Routine formatting, lint, and test fixes.


## [v0.3.4] - 2026-03-20

### Bug Fixes

- Unblocked the daemon event loop so health checks and timers can run between client commands: `handle_connection` now bounds the idle read with a 5 s timeout, and `browser action` drops its daemon connection before the post-dispatch ping probe.
- Fixed `steel profile import` so re-importing an existing profile keeps its original profile id: the command now calls the update endpoint instead of uploading a fresh profile and replacing the id.


## [v0.3.3] - 2026-03-20

### Bug Fixes

- `steel browser start` now surfaces the actual daemon failure (for example, "Invalid API key") instead of a generic "failed to start within Ns" timeout. If the daemon process exits early, the last log line is included in the error and the command returns promptly instead of waiting out the full timeout.

### Miscellaneous

- Added repo toolings: Dependabot config, CI workflow, `cliff.toml`, `deny.toml`, pinned `rust-toolchain.toml`, `rustfmt.toml`, and `typos.toml`.
- Refactored `dispatch_action` in the browser action command by extracting an `into_wire` conversion, collapsing ~570 lines of duplicated match arms.
- Routine dependency updates: `zip` 2.4 → 8.2 ([#48](https://github.com/steel-dev/cli/pull/48)), `rusqlite` 0.33 → 0.39 ([#49](https://github.com/steel-dev/cli/pull/49)), `jiff` 0.1 → 0.2 ([#51](https://github.com/steel-dev/cli/pull/51)), `dialoguer` 0.11 → 0.12 ([#52](https://github.com/steel-dev/cli/pull/52)), `proptest-derive` 0.5 → 0.8 ([#50](https://github.com/steel-dev/cli/pull/50)), plus CI action bumps for `upload-artifact`, `download-artifact`, and `attest-build-provenance` ([#45](https://github.com/steel-dev/cli/pull/45), [#46](https://github.com/steel-dev/cli/pull/46), [#47](https://github.com/steel-dev/cli/pull/47)).


## [v0.3.2] - 2026-03-19

### Security

- Session names are now validated to prevent path traversal, so a crafted `--session` value can no longer escape the daemon's session directory.
- Config writes now go through an atomic, `0600`-permissioned path so credentials in the settings file aren't briefly world-readable during updates.
- Removed `npx @steel-dev/cli` from the steel-browser skill's allowed tools list to keep agents from shelling out to the legacy JS CLI.

### Features

- Implemented a much larger set of browser actions (navigate, click, type, scroll, wait, evaluate, and more) end-to-end across the daemon protocol, engine, and CLI.
- Added `steel browser snapshot diff` to compare two page snapshots.
- Brought `steel browser` subcommands to output parity with each other, covered by a new `tests/output_parity.rs` suite.
- Expanded the `steel-browser` skill with a commands reference and updated SKILL.md guidance.

### Changed

- Profile exports now exclude Chromium's `Preferences` file to avoid carrying over machine-specific state between profiles.
- Profile writes are now atomic (write-to-temp then rename), so a crash mid-write will no longer leave a half-written profile store.

### Bug Fixes

- `steel browser eval --session <name>` now accepts `--session` after the `eval` subcommand, matching the other browser subcommands.

### Miscellaneous

- Hardened daemon socket cleanup so stale sockets from prior runs are removed on startup.
- Switched process liveness checks to `libc::kill(pid, 0)` instead of spawning `kill(1)`.
- Updated `LICENSE`.


## [v0.3.1] - 2026-03-18

### Breaking Changes

- `steel browser start` no longer reattaches to an existing daemon for the same session name — it now stops the old daemon and starts a fresh session. Use `steel browser sessions` to inspect running sessions instead of relying on `start` to reconnect.

### Features

- Added **partial profiles**, allowing targeted selection of which profile components (cookies, storage, etc.) to import/export rather than porting the entire browser profile. ([#43](https://github.com/steel-dev/cli/issues/43))
- Introduced a **daemon-owned session lifecycle**: the browser daemon now owns session state and expiry handling directly, replacing the on-disk `session_state` config layer. ([#44](https://github.com/steel-dev/cli/issues/44))
- Improved profile UX with a better interactive profile selector, cleaner error messages on profile operations, and a dedicated profiles cleanup path.
- `steel install.sh` now detects an existing Node.js-based `@steel-dev/cli` install and removes it automatically before laying down the native binary, so upgrades from the legacy npm CLI no longer leave two `steel` binaries on `PATH`.
- Profile packaging now reports compression progress with the total size (e.g. `Zipping 120 MB (this may take a moment)...`) instead of a silent `Zipping...`.

### Changed

- `install.sh` now passes `STEEL_CLI_NO_MODIFY_PATH=1` to the cargo-dist installer and appends the `~/.steel/bin` PATH entry to the correct shell rc file itself (`.zshrc`, `.bash_profile`/`.bashrc`, fish `conf.d`, or `.profile`), rather than letting the upstream installer edit shell config on its own.
- The npm `postinstall` shim no longer suggests `steel update` when the native binary is already present; it now tells users to `npm uninstall -g @steel-dev/cli` because the npm package is redundant once the native CLI is installed.
- Replaced hand-rolled date/time arithmetic in session handling with a proper `chrono`-based implementation.

### Bug Fixes

- Hardened Brave cookie re-encryption to accept cookies whose `encrypted_value` column is stored as `TEXT` instead of `BLOB`, preventing import failures on Brave profiles.

### Documentation

- Updated README.

### Miscellaneous

- Suppressed the cargo-dist update banner on release builds.


## [v0.3.0] - 2026-03-17

### Breaking Changes

- The CLI has been rewritten from TypeScript/Ink to Rust. The `steel` binary on your PATH continues to work, but the distribution model has changed: releases are now produced by cargo-dist and installed via `install.sh` / Homebrew / prebuilt archives rather than built from Node sources. The npm package still exists as a thin shim (`bin/steel.js` + postinstall) that downloads the prebuilt binary, so `npm i -g @steel-dev/cli` keeps working, but any script that invoked files under `dist/` directly, imported from the package, or depended on the bundled Node runtime will break. Reinstall from the release artifacts and call the `steel` binary instead. ([#41](https://github.com/steel-dev/cli/issues/41))

### Features

- Full Rust rewrite of the CLI with command parity against the previous TypeScript implementation, covering `forge`, `run`, `browser` (start/stop/sessions/live/captcha), `profile`, `credentials`, `dev`, `login`/`logout`, `scrape`, `screenshot`, `pdf`, `cache`, `config`, `settings`, and `update`. Ships with a new daemon-backed browser lifecycle, profile porter, and top-level API client, plus blackbox/compat/lifecycle test suites to lock behavior against the old CLI. ([#41](https://github.com/steel-dev/cli/issues/41))


## [v0.2.4-beta] - 2026-03-17

### Features

- Support more browsers


## [v0.2.2-beta] - 2026-03-14

### Bug Fixes

- *(browser)* Align CDP passthrough and bundle vendored daemon assets

- Test and connect url

- *(browser)* Redact connect URLs and align stealth session payload

- *(browser)* Handle session lookup errors and bootstrap flag parsing

- Don't block commands on failed auto-updates

- Bypass session check for auth/device/session subcommands


### Documentation

- *(plan)* Recalibrate browser fork track statuses

- *(browser)* Publish migration and compatibility references

- *(plan)* Reconcile remaining release gates with completed work

- *(browser)* Expand agent troubleshooting and output security references

- *(plan)* Detail expanded cloud e2e command-chain coverage

- *(skills)* Add steel-browser skill package

- *(steel-browser)* Rewrite skill docs with broader triggers, command cheatsheet, and quick-start patterns


### Features

- *(browser)* Add cloud session lifecycle and passthrough routing

- *(dev)* Move local runtime orchestration into dev namespace

- *(browser)* Add self-hosted endpoint targeting and local runtime install flow

- *(cli)* Finalize browser docs/ux and top-level API tools

- *(browser)* Add manual captcha solve command and update skill docs

- *(browser)* Add captcha status command and named session targeting

- Parallel sessions

- Don't auto update

- Add credentials management commands

- Profiles ([#39](https://github.com/steel-dev/cli/issues/39))


### Miscellaneous

- *(browser)* Package vendored runtimes with multi-os smoke checks

- *(browser)* Enforce playwright-core-only runtime dependencies

- *(cli)* Remove legacy scaffolding and unused dependencies

- Checkpoint local browser and integration changes

- *(cli)* Update version

- Enable shell flag for windows


### Performance

- *(browser)* Add passthrough latency benchmark harness


### Testing

- *(browser)* Expand cloud command-chain coverage and harden harness


## [v0.1.9-beta] - 2025-10-29

### Bug Fixes

- Let publish be manually invoked ([#34](https://github.com/steel-dev/cli/issues/34))

- Build before publish ([#35](https://github.com/steel-dev/cli/issues/35))


## [v0.1.8-beta] - 2025-10-29

### Miscellaneous

- Test npm publish ([#33](https://github.com/steel-dev/cli/issues/33))


## [v0.1.7-beta] - 2025-10-29

### Bug Fixes

- Update npm publish script ([#32](https://github.com/steel-dev/cli/issues/32))


## [v0.1.6-beta] - 2025-10-29

### Bug Fixes

- Update minor wording for release script ([#30](https://github.com/steel-dev/cli/issues/30))


## [v0.1.5-beta] - 2025-10-28

### Bug Fixes

- Let cli vars overwrite system ([#29](https://github.com/steel-dev/cli/issues/29))


## [v0.1.4-beta] - 2025-10-28

### Bug Fixes

- Fix magnitude run and anthropic key issues ([#25](https://github.com/steel-dev/cli/issues/25))

- Add error message ([#27](https://github.com/steel-dev/cli/issues/27))


### Miscellaneous

- Update package version ([#28](https://github.com/steel-dev/cli/issues/28))


## [v0.1.2-beta] - 2025-08-28

### Bug Fixes

- Update manifest version ([#24](https://github.com/steel-dev/cli/issues/24))


## [v0.1.1-beta] - 2025-08-27

### Bug Fixes

- Browser start uses main compose file

- Manifest version + package version ([#23](https://github.com/steel-dev/cli/issues/23))


### Documentation

- Update templates


## [v0.1.0-beta] - 2025-07-30

### Features

- Bump manifest version


## [v0.0.3-beta.1] - 2025-07-30

### Bug Fixes

- Oai-cua path

- Styling on run + ask for task

- Update commands

- Cleanup

- Callout for forge success

- Version bump

- Directories and warnings

- Open browser before

- Updated release script for cli

- Better logs on dependency errors and cleaner callout post forge

- Open window always with -o

- Resolve loading state bug


### Features

- Responsive welcome message

- Auto-update

- Remove jest

- Added setup.sh and homebrew install

- Integrate registry

- Bump manifest version

- Use shorthand as command ([#20](https://github.com/steel-dev/cli/issues/20))


### Miscellaneous

- Version bump

- Package bump

- Remove .DS_Store ([#21](https://github.com/steel-dev/cli/issues/21))


### Nit

- Restore old command


### Update

- Readme


## [v0.0.2-beta] - 2025-07-21

### Bug Fixes

- Some run issues

- Some conditional logic

- Small styling stuff

- Callouts everywhere

- Steel browser errors

- Commands + useprod for api

- Logging + errors

- Update browser start + stop commands

- More styling + logging

- More visibility

- Default api url

- Create dir on setSettings

- Env var fix


### Features

- Removed api-endpoint commands

- Some menu changes, added help for templates, created automatically generated cli reference docs + test scaffolding

- More error logging


## [v0.0.1-beta] - 2025-07-19

### Bug Fixes

- Updated get and post dasboard, made hooks and adding more routes

- Moving config and defining functions to execute

- Added recast to do AST searches

- Update more AST functions and move towards production in terms of code

- Updated cookbook sequence and fixed the project name component

- Added loading for writing directory correctly

- Fixed local start and stop for development

- Added help descriptions for all services

- Update README

- Update README

- Update README

- Refactored lazy api to use one data source and pull params and body from it

- Added zod typing and args for all necessary args

- Unifying examples/adding forge run commands

- Update main command, working on session_id and running

- Getting closer on run command

- Update README.md

- Update version for release

- Update more package stuff

- Updated pastel version

- Updated version and description

- Update build/modules for publish

- Fixed loading on deps and starting local steel with args for run and browser start

- Got cli back to working, added examples

- Run command without dir copy, cache cleaning

- Update examples, fix forge envvar

- Fumbling around with node

- Fixed typescript execution

- Added credentials and magnitude to examples

- Fix contributors

- Update contributors description

- Login/logout flow updated

- Resolve type errors

- Remove deprecated parsing/login flow

- Prettier issues

- Differences


### Features

- Init pastel cli tool, added api and login authentication

- Added a post dashboard and a get dashboard that searches, scrolls

- Added all routes and added functionality for params

- Working on integration functionality

- Add integrations for various libraries with steel.dev

- Added examples for integrating with cookbook

- Updated hook logic and thinking of better ways to abstract

- Cookbook is coming along, getting animations and flow is harder than expected

- Updated components and flow

- Fixed cookbook flow

- Move examples out of src

- Got cookbook working

- Added recast for AST for javascript

- Removed integrate while in beta, added in info, and fixed login to grab API key

- Added settings page and ability to change if a user queries locally or to the cloud

- Added ability to change API calls from local to cloud

- The cli has returned!

- Upddatds login, working on forge/run, updated start, docs, star, support commands

- Getting forge and run working

- Run and forge work, plus extra commands, it is beautiful

- New components

- Implement new login flow

- Custom help message in progress

- Help message/working on success/error/warning components
