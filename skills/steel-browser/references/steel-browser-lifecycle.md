# Steel Browser Lifecycle and Modes

Use this reference when planning session lifecycle, mode selection, and endpoint behavior.

## Command ownership

Steel-owned lifecycle commands:

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`

All other `steel browser <command>` operations are inherited passthrough behavior backed by the vendored `agent-browser` runtime.

## Mode selection

- Cloud mode is default.
- Self-hosted mode is used when `--local` or `--api-url <url>` is provided.
- Keep one mode for an entire workflow unless the user explicitly asks to switch.

## Endpoint precedence

Self-hosted endpoint precedence:

1. `--api-url <url>`
2. `STEEL_BROWSER_API_URL`
3. `STEEL_LOCAL_API_URL`
4. `browser.apiUrl` in `~/.config/steel/config.json`
5. `http://localhost:3000/v1`

Cloud endpoint precedence:

1. `STEEL_API_URL`
2. `https://api.steel.dev/v1`

## `steel browser start` contract

Purpose: create or attach a session.

Main flags:

- `--local`
- `--api-url <url>`
- `--session <name>`
- `--stealth`
- `--proxy <url>`

Parse these output fields:

- `id`: stable session identifier
- `mode`: execution mode
- `name`: session alias if provided
- `live_url`: live-view URL when available
- `connect_url`: display-safe URL with sensitive values redacted

Use `id` for stable machine parsing. Treat `connect_url` as display metadata, not a raw credential.

## `steel browser stop` contract

Purpose: stop active sessions.

Main flags:

- `--all`
- `--local`
- `--api-url <url>`

## `steel browser sessions` contract

Purpose: list session metadata as JSON.

Main flags:

- `--local`
- `--api-url <url>`
- `--raw`

`connectUrl` values are display-safe and redact sensitive query values.

## `steel browser live` contract

Purpose: print the active session live-view URL.

Main flags:

- `--local`
- `--api-url <url>`

## Passthrough bootstrap behavior

For inherited commands, Steel may inject resolved `--cdp` automatically.

- If command already includes `--cdp`: passthrough unchanged.
- If command includes `--auto-connect`: passthrough unchanged.
- If both are present: fail fast.

## Recommended lifecycle pattern

```bash
SESSION="job-$(date +%s)"
steel browser start --session "$SESSION"
steel browser open https://example.com --session "$SESSION"
steel browser snapshot -i --session "$SESSION"
steel browser stop --session "$SESSION"
```

Use a stable session name to avoid accidental session churn across multi-step workflows.
