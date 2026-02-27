# Steel Browser Reference (Commands + Modes)

This reference defines the `steel browser` command surface, including Steel-owned lifecycle commands and inherited `agent-browser` commands.

## Command Ownership

### Steel-Owned Commands

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`

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

Output fields:

- `id`
- `mode`
- `name` (when set)
- `live_url` (when available)
- `connect_url` (when available)

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

### `steel browser live`

Purpose: print current active session live-view URL.

Main flags:

- `--local`
- `--api-url <url>`

## Passthrough Bootstrap Rules

For inherited commands, Steel bootstrap injects a resolved `--cdp` endpoint unless explicit attach flags are present.

- If `--cdp` is present: passthrough unchanged.
- If `--auto-connect` is present: passthrough unchanged.
- If both are present: fail fast.

## Local Runtime UX

Localhost self-hosted flows provide actionable errors:

- Runtime missing: instruct `steel dev install`.
- Runtime installed but unavailable: instruct `steel dev start`.

## Related Docs

- [../browser-compat.md](../browser-compat.md)
- [../migration-agent-browser.md](../migration-agent-browser.md)
- [../upstream-sync.md](../upstream-sync.md)
- [../cli-reference.md](../cli-reference.md)
