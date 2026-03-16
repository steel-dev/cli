# Migration from `agent-browser` to `steel browser`

Use this reference when users bring existing upstream scripts or habits.

## Core migration rule

Replace the command prefix:

- Before: `agent-browser <command> ...`
- After: `steel browser <command> ...`

Steel keeps the same CLI interface as agent-browser and adds lifecycle controls.

## Command mapping

Most commands work with a direct prefix swap. Key equivalences:

| agent-browser             | steel browser                             |
| ------------------------- | ----------------------------------------- |
| `open <url>`              | `open <url>` (alias for `navigate`)       |
| `snapshot`                | `snapshot` (both default to all elements) |
| `snapshot -i`             | `snapshot -i` (interactive-only)          |
| `get text @e1`            | `get text @e1`                            |
| `get html @e1`            | `get html @e1`                            |
| `get url`                 | `get url`                                 |
| `is visible @e1`          | `is visible @e1`                          |
| `tab new`                 | `tab new`                                 |
| `tab 2`                   | `tab switch 2`                            |
| `scroll down 500`         | `scroll down 500`                         |
| `wait @e1`                | `wait --selector @e1`                     |
| `wait 2000`               | `wait --timeout 2000`                     |
| `wait --load networkidle` | `wait --load-state networkidle`           |
| `wait --fn "expr"`        | `wait --function "expr"`                  |
| `screenshot ./page.png`   | `screenshot -o ./page.png`                |
| `screenshot --full`       | `screenshot --full-page`                  |

## Snapshot flag differences

Snapshot flags now match agent-browser 1:1:

| agent-browser       | steel browser          |
| ------------------- | ---------------------- |
| `-i, --interactive` | `-i, --interactive`    |
| `-c`                | `-c, --compact`        |
| `-d <n>`            | `-d, --max-depth <n>`  |
| `-s <sel>`          | `-s, --selector <sel>` |

## Example conversion

```bash
# Before
agent-browser open https://example.com
agent-browser snapshot -i
agent-browser click @e3
agent-browser get text @e7
agent-browser wait --load networkidle
agent-browser screenshot ./page.png

# After
steel browser start
steel browser open https://example.com
steel browser snapshot -i
steel browser click @e3
steel browser get text @e7
steel browser wait --load-state networkidle
steel browser screenshot -o ./page.png
steel browser stop
```

## Steel-only commands

These are Steel additions not present in agent-browser:

- `steel browser start` — create/attach a cloud browser session
- `steel browser stop` — release the session
- `steel browser sessions` — list active sessions
- `steel browser live` — open the live session viewer
- `steel browser captcha status/solve` — CAPTCHA management

## Not yet implemented

These agent-browser commands are not yet available in Steel:

- `set viewport/device/geo/offline/headers/credentials/media` — browser configuration
- `cookies get/set/clear` — cookie management
- `storage local/session` — web storage
- `network route/unroute/requests` — network interception
- `find role/text/label/...` — semantic locators (Steel `find` uses CSS selectors only)
- `dialog accept/dismiss` — dialog handling
- `upload`, `drag` — file upload, drag and drop
- `pdf` — PDF export
- `console`, `errors` — debug log capture
- `frame`, `window new` — frame/window management
- `keydown`, `keyup` — low-level keyboard input
- `record start/stop` — video recording

Use `steel browser --help` to see all currently available commands.
