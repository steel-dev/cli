# Agent Troubleshooting Playbook (`steel browser`)

This playbook is optimized for agents and automation loops that invoke
`steel browser` commands repeatedly.

## Recommended Session Pattern

Use a named session and keep it constant for the whole workflow:

```bash
steel browser start --session "$SESSION_NAME"
steel browser open https://example.com --session "$SESSION_NAME"
steel browser snapshot -i --session "$SESSION_NAME"
steel browser stop --session "$SESSION_NAME"
```

Why:

- avoids accidental new sessions between commands.
- allows cross-process reattach with the same `--session <name>`.
- keeps recovery steps predictable.

## Parsing Rules for Automation

From `steel browser start` output:

- `id`: canonical session identifier.
- `mode`: `cloud` or `local`.
- `name`: session alias (when provided).
- `live_url`: viewer URL (when available).
- `connect_url`: display-safe URL; sensitive query values are redacted.

Do not assume `connect_url` contains usable raw credentials.
Likewise, `steel browser sessions` returns display-safe `connectUrl` fields.
It emits compact metadata by default; add `--raw` for full session payloads.

## Building a Full CDP URL Safely

If a downstream tool requires an auth-bearing CDP URL, construct it from
`id` + environment key:

```bash
wss://connect.steel.dev?sessionId=<SESSION_ID>&apiKey=$STEEL_API_KEY
```

This keeps secrets in env/config rather than CLI logs.

## Fast Diagnostics

Check active context:

```bash
steel browser sessions
steel browser live
```

Cloud auth verification:

```bash
steel config
```

Self-hosted runtime verification:

```bash
steel dev start
steel browser start --local --session "$SESSION_NAME"
```

## Common Failures and Actions

- `Missing browser auth...`:
  authenticate (`steel login`) or export `STEEL_API_KEY`.
- `Failed to reach Steel session API ...`:
  check endpoint flags/env and runtime availability.
- Existing session not reused:
  verify exact `--session` spelling and same mode (cloud/local).
- Stale state:
  `steel browser stop --all`, then start a new named session.

## PR/CI Hygiene for Agent Logs

- never print `STEEL_API_KEY` directly.
- treat `connect_url` as non-secret display output only.
- prefer session IDs and named sessions in logs for reproducible debugging.
