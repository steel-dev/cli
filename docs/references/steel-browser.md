# Steel Browser Reference (Commands + Modes)

This reference defines the `steel browser` command surface, including Steel-owned lifecycle commands and inherited `agent-browser` commands.

## Command Ownership

### Steel-Owned Commands

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`
- `steel browser captcha solve`

### Inherited Commands (Vendored agent-browser Runtime)

All other `steel browser <command>` invocations are delegated to vendored upstream runtime behavior.

Upstream command catalog (pinned reference):

- Local transformed command catalog for Steel context:
  - `./steel-browser-commands.md`
- https://github.com/vercel-labs/agent-browser/blob/b59dc4c82c0583b60849d110f27982fac85f1a07/cli/README.md
- https://github.com/vercel-labs/agent-browser/tree/b59dc4c82c0583b60849d110f27982fac85f1a07/skills/agent-browser/references
- https://github.com/vercel-labs/agent-browser/blob/b59dc4c82c0583b60849d110f27982fac85f1a07/skills/agent-browser/references/commands.md

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

Purpose: create or attach a session and print stable fields.

Main flags:

- `--local`
- `--api-url <url>`
- `--session <name>`
- `--stealth`
- `--proxy <url>`
- `--session-solve-captcha`

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
- `--stealth` and `--proxy` are create-time flags. If `--session <name>` attaches to
  an existing live session, these values are not re-applied.

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

### `steel browser stop`

Purpose: stop active session(s).

Main flags:

- `--all`
- `--local`
- `--api-url <url>`

### `steel browser sessions`

Purpose: list sessions as JSON.

Main flags:

- `--local`
- `--api-url <url>`
- `--raw`

Output note:

- `connectUrl` values in JSON are display-safe and redact sensitive query values.
- Default output is compact and omits each session's full raw payload.
- Use `--raw` when you need the full underlying API payload.

### `steel browser live`

Purpose: print current active session live-view URL.

Main flags:

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

## Passthrough Bootstrap Rules

For inherited commands, Steel bootstrap injects a resolved `--cdp` endpoint unless explicit attach flags are present.

- If `--cdp` is present: passthrough unchanged.
- If `--auto-connect` is present: passthrough unchanged.
- If both are present: fail fast.

## Local Runtime UX

Localhost self-hosted flows provide actionable errors:

- Runtime missing: instruct `steel dev install`.
- Runtime installed but unavailable: instruct `steel dev start`.

## Troubleshooting (Agent-Focused)

- `Missing browser auth. Run steel login or set STEEL_API_KEY.`:
  run `steel login` locally, or set `STEEL_API_KEY` in CI/job env.
- `Failed to reach Steel session API ...` in local mode:
  verify `steel dev start` is running and endpoint resolution matches expected
  host/port.
- Commands open a fresh browser unexpectedly:
  ensure all steps share the same `--session <name>` and mode (`cloud` vs
  `--local`/`--api-url`).
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

- [../browser-compat.md](../browser-compat.md)
- [../migration-agent-browser.md](../migration-agent-browser.md)
- [./agent-troubleshooting.md](./agent-troubleshooting.md)
- [../upstream-sync.md](../upstream-sync.md)
- [../cli-reference.md](../cli-reference.md)
