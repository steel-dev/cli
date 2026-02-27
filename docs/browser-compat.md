# Steel Browser Compatibility

This document tracks compatibility between `steel browser` and upstream `agent-browser` command behavior.

## Compatibility Summary

- Steel-owned commands: `start`, `stop`, `sessions`, `live`
- Inherited commands: all non-lifecycle `steel browser <command>` routes to vendored upstream runtime
- Default mode: Steel cloud session bootstrap
- Self-hosted mode: `--local` or `--api-url <url>`

## Behavior Matrix

| Area | Status | Notes |
|------|--------|-------|
| Passthrough command execution | Compatible | stdout/stderr + exit codes are forwarded from runtime process |
| Inherited command names and flags | Compatible | delegated to vendored runtime |
| Help for inherited commands | Compatible | delegated help output |
| Cloud session bootstrap | Steel extension | automatic session create/attach before passthrough actions |
| `start/stop/sessions/live` | Steel extension | native lifecycle UX on top of Steel session API |
| Explicit endpoint targeting (`--api-url`) | Steel extension | implies self-hosted mode |
| Local runtime lifecycle (`dev install/start/stop`) | Steel extension | explicit install/start/stop boundaries |
| Explicit attach flags (`--cdp`, `--auto-connect`) | Compatible | bypass Steel bootstrap injection |

## Inherited Command Families

Inherited command groups remain upstream-driven:

- Navigation (`open`, `back`, `forward`, `reload`, `close`)
- Interaction (`click`, `fill`, `type`, `press`, `hover`, `select`, `check`)
- Retrieval (`get text`, `get url`, `get title`, and related getters)
- Snapshot and screenshot (`snapshot`, `screenshot`, `diff`)
- Wait/debug/network/storage/device commands supported by upstream runtime

For exhaustive command and flag coverage, use upstream command references:

- https://github.com/vercel-labs/agent-browser/blob/b59dc4c82c0583b60849d110f27982fac85f1a07/cli/README.md
- https://github.com/vercel-labs/agent-browser/blob/b59dc4c82c0583b60849d110f27982fac85f1a07/skills/agent-browser/references/commands.md

## Known Steel-Specific Differences

1. Browser command path bypasses Steel auto-update checks.
2. Localhost self-hosted errors provide actionable `dev install` / `dev start` guidance.
3. Session state is persisted in `~/.config/steel/browser-session-state.json` for Steel lifecycle operations.

## Deferred Mappings (Tracked Post-V1)

The following areas are intentionally deferred and tracked as compatibility follow-ups:

- profile mapping semantics (`--profile` behavior vs Steel session persistence)
- upload endpoint materialization and policy
- download endpoint materialization and policy

## Validation Targets

- `npm run test:unit`
- `npm run test:integration`
- `npm run browser:cloud:smoke` (credentialed env)
