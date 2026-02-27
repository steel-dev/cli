# Migration Guide: `agent-browser` -> `steel browser`

This guide covers command-prefix migration from upstream `agent-browser` to `steel browser`.

## Migration Goal

For most automation scripts, replace command prefix only:

- before: `agent-browser <command> ...`
- after: `steel browser <command> ...`

Steel keeps inherited command behavior via vendored runtime passthrough and adds Steel-native session lifecycle commands.

## Quick Migration Steps

1. Install/upgrade Steel CLI.
2. Authenticate once with `steel login` (or set `STEEL_API_KEY` in CI).
3. Replace command prefix in scripts.
4. Run smoke flow: `start -> open -> snapshot -i -> stop`.
5. If self-hosted, set endpoint explicitly with `--api-url`.

## Example Script Diff

```bash
# Before
agent-browser open https://example.com
agent-browser snapshot -i
agent-browser click @e3
agent-browser get text @e7

# After
steel browser open https://example.com
steel browser snapshot -i
steel browser click @e3
steel browser get text @e7
```

## Steel-Native Lifecycle Commands

Steel adds lifecycle/session helpers that are not direct upstream command replacements:

- `steel browser start`
- `steel browser stop`
- `steel browser sessions`
- `steel browser live`

Use these when you need explicit create/stop/list/live workflows.

## Auth Model Differences

### Cloud (default)

- Preferred: run `steel login` once.
- CI fallback: set `STEEL_API_KEY`.

### Self-hosted

Use one of:

- `--api-url <url>` (recommended for deterministic runs)
- `--local` plus env/config endpoint resolution

## Endpoint Resolution for Self-Hosted Runs

1. `--api-url <url>`
2. `STEEL_BROWSER_API_URL`
3. `STEEL_LOCAL_API_URL`
4. `browser.apiUrl` in `~/.config/steel/config.json`
5. `http://localhost:3000/v1`

## Localhost Runtime Flow

For local Docker runtime workflows:

```bash
steel dev install
steel dev start
steel browser start --local --session local-job
steel browser open https://example.com --session local-job
steel browser stop
steel dev stop
```

If runtime assets are missing, browser local-mode commands instruct `steel dev install`.
If runtime is installed but unavailable, they instruct `steel dev start`.

## Attach Flag Behavior

When explicit attach flags are provided, Steel passthrough does not inject bootstrap flags:

- `--cdp <url|port>`: forwarded unchanged
- `--auto-connect`: forwarded unchanged

`--cdp` and `--auto-connect` cannot be combined.

## Output Security Contract

`steel browser start` and `steel browser sessions` print display-safe connect
URLs. Sensitive query values like `apiKey` are redacted in CLI output to avoid
leaking credentials into logs.

For tools that need a fully-authenticated CDP URL, compose it from session `id`
and `STEEL_API_KEY` in your runtime environment.

## Compatibility References

- [Browser Compatibility Matrix](./browser-compat.md)
- [Steel Browser Reference](./references/steel-browser.md)
- Upstream commands catalog:
  - https://github.com/vercel-labs/agent-browser/blob/b59dc4c82c0583b60849d110f27982fac85f1a07/skills/agent-browser/references/commands.md
