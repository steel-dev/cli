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

From action output (`--json`, or any piped output):

- `success`: whether the command did what it was asked.
- `error_code`: the engine error code on failure, such as `stale_ref`.
- `data.page`: the tab, URL, and title the command used.
- `data.results`: one entry per command, with fields such as `snapshot` or `value`.
- `data.notices`: dialogs, new tabs, downloads, and replays to act on.

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
# or: steel browser live --session "$SESSION_NAME"
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
- `Session "<name>" is not running...`:
  the session expired or the name is misspelled. Check `steel browser sessions`,
  then run `steel browser start --session <name>`.
- `session_ended`: the session was released, often by the 10-minute inactivity
  timeout. Start a new one and open the page again.
- `stale_ref`: take a new snapshot and use the new refs.
- `tab_required`: several tabs are open. Run `steel browser tab list`, then
  `steel browser tab switch <tab>`.
- `dialog_pending`: answer with `steel browser dialog accept` or `dismiss`.
- `delivery_failed`: the API could not reach the browser. Run the same command
  again; it will not run twice.
- Stale state:
  `steel browser stop --all`, then start a new named session.

## PR/CI Hygiene for Agent Logs

- never print `STEEL_API_KEY` directly.
- treat `connect_url` as non-secret display output only.
- prefer session IDs and named sessions in logs for reproducible debugging.
