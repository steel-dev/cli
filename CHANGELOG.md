# Changelog

## [0.5.0]

Stable release of the functionality shipped in `v0.5.0-preview.9`.

### Agent setup and authentication

- Sign in through first-party device authorization. Agent setup presents an
  approval URL and resumes the saved authorization after the user approves it.
- Target skill installation explicitly with `steel init --agent --from <client>`.
  Supported targets are `opencode`, `codex`, `pi`, and `claude-code`.
- Persist the selected setup client across approval/resume, reuse readable
  installed skills, and report targeted skill-installation failures.
- Distinguish CLI/auth/API setup from skill loading and first-task verification.
- Keep connection credentials out of displayed session metadata and the Steel
  API key out of the skill-installer child environment.

### Browser automation

- Execute browser actions through the HTTP Commands API, without the previous
  local browser daemon or `agent-browser` dependency.
- Retrieve browser downloads to a local file with
  `steel browser wait --download --output <path>`.
- Preserve quoting in browser batches, improve profile import fidelity, and
  report when starting a named session replaces an existing session.
- Add session metadata and signal filters.
- Recognize stable releases as newer than their prerelease versions.

### Upgrade notes

- Output is text by default, including when piped. Scripts that parse JSON must
  pass `--json`; each command emits one JSON result.
- Use `steel browser stop` in place of the former `browser close` command.
  Review `steel browser start --help` for the current create-time flags.
- Stopping an ID-attached browser session forgets its local attachment without
  releasing the cloud session. `browser stop --all` also leaves attached cloud
  sessions running. Use an explicit `steel sessions release` target when release
  is intended; account-wide release requires `--all`.
- `sessions agent-logs --follow` is no longer available. Computer, checkpoint,
  secret, and environment commands from earlier 0.5 previews have been removed.
- `STEEL_ONBOARDING_FROM` remains telemetry metadata. Pass `--from` directly to
  `steel init --agent` to select the skill-installation target.

See the [CLI reference](docs/cli-reference.md) for current commands and flags.

[0.5.0]: https://github.com/steel-dev/cli/compare/v0.4.4...v0.5.0
