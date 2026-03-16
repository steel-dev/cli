# Migration from `agent-browser` to `steel browser`

Use this reference when users bring existing upstream scripts or habits.

## Core migration rule

Replace the command prefix:

- Before: `agent-browser <command> ...`
- After: `steel browser <command> ...`

Steel keeps the same CLI interface as agent-browser and adds lifecycle controls.

## Differences

Most commands work with a direct prefix swap. These are the exceptions:

| agent-browser           | steel browser              | reason              |
| ----------------------- | -------------------------- | ------------------- |
| `tab 2`                 | `tab switch 2`             | positional → subcmd |
| `wait @e1`              | `wait --selector @e1`      | positional → flag   |
| `wait 2000`             | `wait --timeout 2000`      | positional → flag   |
| `screenshot ./page.png` | `screenshot -o ./page.png` | positional → flag   |

## Example conversion

```bash
# Before
agent-browser open https://example.com
agent-browser snapshot -i
agent-browser click @e3
agent-browser get text @e7
agent-browser wait --load networkidle
agent-browser screenshot ./page.png

# After — just swap the prefix (aliases handle the rest)
steel browser start
steel browser open https://example.com
steel browser snapshot -i
steel browser click @e3
steel browser get text @e7
steel browser wait --load networkidle
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

Use `steel browser --help` to see all currently available commands.
