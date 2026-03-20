---
name: steel-browser
allowed-tools: Bash(steel:*)
description: >-
  Use this skill for any web task where WebFetch or curl would fail or be
  insufficient — pages that require JavaScript to render, forms to fill and
  submit, screenshots or PDFs of live pages, CAPTCHA/bot-protection bypass,
  login flows, and multi-step browser navigation with persistent session state.
  WebFetch returns empty HTML for JS-rendered pages; this skill runs a real
  cloud browser that executes JavaScript, maintains cookies, clicks buttons,
  and handles anti-bot measures. Trigger when the user wants you to actually
  perform a web task (visit, interact, extract, capture) rather than just write
  code for it. Skip only for: static pages a simple GET can fetch, localhost or
  private-network targets, writing browser automation code the user will run
  themselves, or conceptual questions about browser tools.
---

# Steel Browser Skill

Steel gives agents cloud browser sessions and fast API tools (`scrape`, `screenshot`, `pdf`).

## Choose the right tool first

| Goal | Tool |
|------|------|
| Extract text/HTML from a mostly-static page | `steel scrape <url>` |
| One-shot screenshot or PDF | `steel screenshot <url>` / `steel pdf <url>` |
| Multi-step interaction, login, forms, JS-heavy pages | `steel browser start` + interaction loop |
| Anti-bot / CAPTCHA sites | `steel browser start --stealth` |

Start with `steel scrape` when you only need page content. Escalate to `steel browser` when the page requires interaction or JavaScript rendering.

## Core workflow

1. **Start** a named session (use `--session-timeout 3600000` for tasks over 5 min)
2. **Navigate** to the target URL
3. **Snapshot** to get current page state and element refs
4. **Interact** using refs from the snapshot (click, fill, select, etc.)
5. **Re-snapshot** after every navigation or meaningful DOM change
6. **Verify** state with `wait`, `get`, or another snapshot
7. **Stop** the session when done

```bash
steel browser start --session my-task --session-timeout 3600000
steel browser open https://example.com --session my-task
steel browser snapshot -i --session my-task
steel browser fill @e3 "search term" --session my-task
steel browser click @e7 --session my-task
steel browser wait --load networkidle --session my-task
steel browser snapshot -i --session my-task
steel browser stop --session my-task
```

**RULE: Never use an `@eN` ref without a fresh snapshot. Refs expire after navigation or DOM changes.**

Element refs from `snapshot -i` are more reliable than CSS selectors — always prefer them. Use `snapshot -i -c` for large DOMs, or `-d 3` to limit depth.

Always use the same `--session <name>` on every command in a workflow. The `--session` flag takes the name string you chose, not the UUID from JSON output.

## Essential commands

### Session lifecycle

```bash
steel browser start --session <name> --session-timeout 3600000
steel browser start --session <name> --stealth
steel browser start --session <name> --proxy <url>
steel browser sessions
steel browser live --session <name>
steel browser stop --session <name>
steel browser stop --all
```

### Navigation and inspection

```bash
steel browser open <url> --session <name>
steel browser snapshot                         # full accessibility tree
steel browser snapshot -i                      # interactive elements + refs
steel browser snapshot -c                      # compact output
steel browser snapshot -i -c -d 3             # combine flags
steel browser get url --session <name>
steel browser get title --session <name>
steel browser get text @e1 --session <name>
steel browser back --session <name>
steel browser forward --session <name>
steel browser reload --session <name>
```

### Interaction

```bash
steel browser click @e1 --session <name>
steel browser dblclick @e1 --session <name>
steel browser fill @e2 "value" --session <name>
steel browser type @e2 "value" --delay 50 --session <name>
steel browser press Enter --session <name>
steel browser press Control+a --session <name>
steel browser hover @e1 --session <name>
steel browser click @e1 --session <name>            # prefer click for checkboxes
steel browser select @e1 "option" --session <name>
steel browser scroll down 500 --session <name>
steel browser scrollintoview @e1 --session <name>
steel browser drag @e1 @e2 --session <name>
steel browser tab new --session <name>
steel browser tab switch 2 --session <name>
steel browser tab list --session <name>
steel browser tab close --session <name>
```

### Synchronization

```bash
steel browser wait --load networkidle --session <name>
steel browser wait --selector ".loaded" --state visible --session <name>
steel browser wait -t "Success" --session <name>
steel browser wait -u "/dashboard" --session <name>
```

### Extraction

```bash
steel browser get text @e1 --session <name>
steel browser get html @e1 --session <name>
steel browser get value @e1 --session <name>
steel browser get attr @e1 href --session <name>
steel browser get count ".item" --session <name>
steel browser content --session <name>
steel browser eval "document.querySelectorAll('a').length" --session <name>
steel browser find ".item" --session <name>
```

### Screenshots and capture

```bash
steel browser screenshot -o ./page.png --session <name>
steel browser screenshot --full --session <name>
steel browser screenshot --selector "#chart" --session <name>
```

Top-level `steel screenshot <url>` and `steel pdf <url>` are stateless one-shot API calls — they do not take `--session`, `--stealth`, or `-o` flags. Use `steel browser screenshot` for in-session captures.

### Cookies and storage

```bash
steel browser cookies --session <name>                              # list all cookies
steel browser cookies set <name> <value> --session <name>           # set a cookie
steel browser cookies set <name> <value> --domain .example.com      # with domain
steel browser cookies clear --session <name>                        # clear all cookies

steel browser storage local --session <name>                        # get all localStorage
steel browser storage local <key> --session <name>                  # get one key
steel browser storage local set <key> <value> --session <name>      # set a value
steel browser storage local clear --session <name>                  # clear localStorage
steel browser storage session --session <name>                      # sessionStorage (same API)
```

### Browser settings

```bash
steel browser set viewport 1920 1080 --session <name>
steel browser set geo 37.7749 -122.4194 --session <name>
steel browser set offline on --session <name>
steel browser set useragent "Custom UA" --session <name>
```

Note: `steel browser set headers` panics in steel 0.3.2 (clap bug). Use `eval` for header injection.

### CAPTCHA

```bash
steel browser start --session <name> --stealth
steel browser captcha status --wait --session <name>
steel browser captcha solve --session <name>
```

## eval for capability gaps

Use `steel browser eval "<js>" --session <name>` when no direct command exists. Common uses: network interception (fetch monkey-patch), cookie manipulation beyond the `cookies` command, DOM state injection, complex form widgets (React date pickers, autocomplete inputs).

For detailed eval patterns (drag fallback, network interception, file upload injection), see [references/steel-browser-commands.md](references/steel-browser-commands.md).

## Commands that do NOT exist

Do not attempt these. They will fail and waste turns.

| Does NOT exist | Use instead |
|---|---|
| `steel browser record` / `video` | No recording; use `steel browser live` for viewer URL |
| `steel browser triple-click` | `press Control+a` or `eval` with `.select()` |
| `steel browser network` / `route` | `eval` with fetch monkey-patch |
| `steel browser console` / `errors` | `eval` with console interceptor |
| `steel browser frame` | `eval` with iframe contentDocument |
| `steel browser tabs` (plural) | `steel browser tab list` (singular `tab`) |
| `steel browser execute` / `run` | `steel browser eval` |
| `steel browser is-visible` | `steel browser is` |
| `steel browser set device` | `set viewport` + `set useragent` separately |
| `steel browser resize` | `steel browser set viewport W H` |
| `steel browser geolocation` | `steel browser set geo LAT LON` |
| `steel browser pdf` | Top-level `steel pdf <url>` (no browser session needed) |

## Troubleshooting

```bash
steel browser sessions
steel browser live --session <name>
```

| Symptom | Fix |
|---------|-----|
| `Missing browser auth` | `steel login` or set `STEEL_API_KEY` |
| `No running session "<name>"` | Check spelling; run `steel browser stop --all` then restart with same name |
| Session not reused between commands | Use exact same `--session <name>` on every command |
| CAPTCHA blocking | `steel browser captcha status --wait`; restart with `--stealth` |
| Stale session / stuck state | `steel browser stop --all` then fresh named session |
| `steel: command not found` | `curl -LsSf https://setup.steel.dev \| sh` and add `$HOME/.steel/bin` to PATH |

Full playbook: [references/troubleshooting.md](references/troubleshooting.md).

## Reference routing

- Full command reference: [references/steel-browser-commands.md](references/steel-browser-commands.md)
- Lifecycle, endpoints, CAPTCHA modes: [references/steel-browser-lifecycle.md](references/steel-browser-lifecycle.md)
- Migration from agent-browser: [references/migration-agent-browser.md](references/migration-agent-browser.md)
- Error recovery playbooks: [references/troubleshooting.md](references/troubleshooting.md)
