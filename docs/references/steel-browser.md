# Steel Browser Reference (Commands + Modes)

This reference defines how `steel browser` runs and the contracts of its session commands.

## How Commands Run

Each `steel browser` action is one HTTP request to the session's commands API:

- `POST /v1/sessions/{sessionId}/commands` with the API key, a random `Idempotency-Key`, and `after` (the last operation id the CLI received).
- The browser engine runs inside the session next to Chrome. It holds refs, tabs, dialogs, and the execution records.
- The CLI keeps no process running. It stores per-session state in `~/.config/steel/browser-sessions/<name>.json`.

The full command list is in [./steel-browser-commands.md](./steel-browser-commands.md).

## Modes

- Cloud mode (default): no mode flag required.
- Self-hosted mode: use `--local` or `--api-url <url>`.

`--api-url` implies self-hosted mode.

## Endpoint Resolution

Self-hosted endpoint precedence:

1. `--api-url <url>`
2. `STEEL_BROWSER_API_URL`
3. `STEEL_LOCAL_API_URL`
4. `browser.apiUrl` in `~/.config/steel/config.json`
5. `http://localhost:3000/v1`

Cloud endpoint precedence:

1. `STEEL_API_URL`
2. `https://api.steel.dev/v1`

## Steel-Owned Command Contracts

### `steel browser start`

Purpose: create a session and print stable fields. If the name already has a session, the old one is released first.

Main flags:

- `--local`
- `--api-url <url>`
- `--session <name>`
- `--stealth`
- `--proxy <url>`
- `--session-timeout <ms>`
- `--inactivity-timeout <ms>` (default 600000; `0` turns it off)
- `--session-solve-captcha`
- `--namespace <name>`
- `--credentials`
- `--profile <name>`
- `--update-profile`

Flag semantics:

- `--stealth` applies a session-creation preset:
  - `stealthConfig.humanizeInteractions = true`
  - `stealthConfig.autoCaptchaSolving = true`
  - `solveCaptcha = true`
- `--session-solve-captcha` enables manual CAPTCHA solving for new sessions:
  - `solveCaptcha = true`
  - `stealthConfig.autoCaptchaSolving` is not forced on
- `--proxy <url>` sets `proxyUrl` on session creation. The Sessions API may return
  `proxySource: "external"` rather than echoing the proxy URL in responses.
- `--namespace <name>` sets the credential namespace for the session. Credentials
  stored under this namespace will be available for injection.
- `--credentials` enables credential injection for the session. Sends
  `credentials: {}` in the session creation payload.
- `--profile <name>` loads a previously imported browser profile (cookies, local
  storage, etc.) into the session.
- `--update-profile` saves session state back to the profile when the session ends.
  Requires `--profile`.
- `--inactivity-timeout <ms>` releases the session after this long without
  commands, CDP messages, or live-view input. Commands count as activity.

Output fields:

- `id`
- `mode`
- `name` (when set)
- `live_url` (when available)
- `connect_url` (when available)

Security contract:

- `connect_url` is safe for logs and transcripts. Sensitive query values
  (`apiKey`, `token`, and aliases) are redacted as `REDACTED`.
- If a workflow needs a fully-authenticated CDP URL, combine the session `id`
  with `STEEL_API_KEY` instead of scraping secrets from CLI output.

Agent parsing contract:

- Parse `id` as the stable handle for follow-up commands.
- Treat `connect_url` as display metadata, not a secret-bearing credential.
- Prefer `--session <name>` for cross-command and cross-process continuity.
- `--session <session-id>` drives a session created elsewhere (SDK, MCP, dashboard).

### `steel browser stop`

Purpose: stop active session(s).

Main flags:

- `--session <name>`
- `--all`
- `--local`
- `--api-url <url>`

### `steel browser sessions`

Purpose: list the local sessions that are still live.

Output note:

- Each row has `id`, `name`, `mode`, `status`, and `viewerUrl` (JSON).
- Sessions that ended are removed from local state.

### `steel browser live`

Purpose: print current active session live-view URL.

Main flags:

- `--session <name>`
- `--local`
- `--api-url <url>`

### `steel browser captcha solve`

Purpose: manually trigger CAPTCHA solving for a target session.

Main flags:

- `--session-id <id>`
- `--session <name>`
- `--page-id <id>`
- `--url <url>`
- `--task-id <id>`
- `--local`
- `--api-url <url>`
- `--raw`

API mapping:

- Endpoint: `POST /v1/sessions/{sessionId}/captchas/solve`
- Request body: optional `pageId`, `url`, `taskId`
- Response: `success` + optional `message`

## Profile Persistence

Profiles let sessions start with pre-existing browser state (cookies, local storage, IndexedDB, etc.) imported from a local Chrome installation.

Setup (macOS only, run by user):

```bash
steel profile import --name myapp          # import Chrome profile to Steel
steel profile sync --name myapp            # re-sync local changes to Steel
steel profile list                         # list saved profiles
steel profile delete --name myapp          # remove local metadata
```

Usage in sessions:

```bash
steel browser start --session job --profile myapp                # load profile
steel browser start --session job --profile myapp --update-profile  # load + save back
```

Profile metadata is stored at `~/.config/steel/profiles/<name>.json`.

## Local Runtime

Self-hosted steel-browser serves the commands API only when it runs with
`ENABLE_COMMANDS_API=true`. Without it, browser actions fail with
`missing_capability`. `steel dev start` does not turn it on yet.

## Troubleshooting (Agent-Focused)

- `Missing browser auth. Run steel login or set STEEL_API_KEY.`:
  run `steel login` locally, or set `STEEL_API_KEY` in CI/job env.
- `Failed to reach Steel session API ...` in local mode:
  verify `steel dev start` is running and endpoint resolution matches expected
  host/port.
- Commands open a fresh browser unexpectedly:
  ensure all steps share the same `--session <name>`.
- `stale_ref`: take a new snapshot (`steel browser snapshot -i`) and use the new refs.
- `tab_required`: several tabs are open. Pick one with `steel browser tab switch <tab>`.
- `dialog_pending`: answer the dialog with `steel browser dialog accept` or `dismiss`.
- `This is the stored result of an earlier run`: the same command already ran
  after the last response the CLI received. Nothing ran again.
- `delivery_failed`: the API could not reach the browser. Run the same command
  again; it will not run twice.
- `No active live session found` from `steel browser live`:
  run `steel browser start` first, then retry `steel browser live`.
- CAPTCHA solving notes:
  - CAPTCHA solving requires a paid Steel plan.
  - `--stealth` enables auto solving; wait and monitor with screenshots.
  - `--session-solve-captcha` enables manual solving; use `steel browser captcha solve`.
  - Default sessions do not solve captchas; restart with CAPTCHA solving enabled and navigate back.
  - Proxy + CAPTCHA solving together is a strong anti-bot evasion combination.
- Need hard reset of stale session mapping:
  run `steel browser stop --all` and start a fresh named session.

## Related Docs

- [./steel-browser-commands.md](./steel-browser-commands.md)
- [./agent-troubleshooting.md](./agent-troubleshooting.md)
- [../cli-reference.md](../cli-reference.md)
