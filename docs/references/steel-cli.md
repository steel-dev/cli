# Steel CLI Reference (High-Level)

This reference captures the stable, high-level command surface of `steel`.

For generated flags and argument schemas, use [../cli-reference.md](../cli-reference.md).

## Command Groups

### Project and Workflow Commands

- `steel forge`: initialize a project from a Steel template.
- `steel run`: execute a Steel workflow from templates/tasks.

### Browser Commands

- `steel browser start`: create or attach a Steel browser session.
- `steel browser stop`: stop active session(s).
- `steel browser sessions`: list session metadata as JSON.
- `steel browser live`: print the active session live-view URL.
- `steel browser <inherited-command>`: pass through to vendored `agent-browser` runtime.

### Local Runtime Commands

- `steel dev install`: install local runtime assets only.
- `steel dev start`: start local runtime containers.
- `steel dev stop`: stop local runtime containers.

### Account and Utility Commands

- `steel login`: authenticate and save API key in config.
- `steel logout`: clear local auth.
- `steel config`: print current auth/session context.
- `steel settings`: switch persisted local/cloud preference.
- `steel cache`: clean cache.
- `steel update`: update CLI.
- `steel docs`: open docs site.
- `steel support`: open support channel.
- `steel star`: open repository page.

## Global Conventions

- `--help` and `--version` are available globally.
- Browser command paths skip auto-update checks for lower command latency.
- Command help for inherited browser commands is delegated to the vendored browser runtime.

## Core Config Paths

- Config directory: `~/.config/steel`
- Main config: `~/.config/steel/config.json`
- Browser session state: `~/.config/steel/browser-session-state.json`

## Environment Variables (Common)

- `STEEL_API_KEY`: cloud auth for session API and browser runtime.
- `STEEL_API_URL`: cloud API endpoint override.
- `STEEL_BROWSER_API_URL`: canonical self-hosted local endpoint override.
- `STEEL_LOCAL_API_URL`: backward-compatible self-hosted alias.
- `STEEL_CONFIG_DIR`: override config directory root.

## Key References

- [Steel Browser Reference](./steel-browser.md)
- [Browser Compatibility](../browser-compat.md)
- [agent-browser Migration Guide](../migration-agent-browser.md)
