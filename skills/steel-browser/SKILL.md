---
name: steel-browser
allowed-tools:
  - "Bash(steel:*)"
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

# Steel

> Cloud browser infrastructure for AI agents. Steel gives your agent a real browser that can navigate pages, fill forms, solve CAPTCHAs, and extract content.

If Steel is not installed yet, execute the setup steps below autonomously. Only ask the user to do one thing: click the authentication link.

OBJECTIVE: Install the Steel CLI, authenticate, and verify everything works.

DONE WHEN: `steel scrape https://example.com` returns page content successfully.

## Setup

### Step 1: Install Steel CLI

Check if `steel` is already available:

```bash
steel --version
```

If the command is not found, install it:

```bash
curl -sSf https://setup.steel.dev/install.sh | sh
export PATH="$HOME/.steel/bin:$PATH"
```

### Step 2: Authenticate

If `STEEL_API_KEY` is already set in the environment, skip this step.

Otherwise, run:

```bash
steel login
```

This will print a URL. **Ask the user to click the link to complete authentication.** Wait for the command to finish — it will print "Authentication successful" when the user has approved.

### Step 3: Verify

```bash
steel scrape https://example.com
```

This should return Markdown content. If it does, setup is complete.

---

## Choose the right tool

| Task | Tool |
|------|------|
| Extract text/HTML from a page | `steel scrape <url>` |
| Take a screenshot | `steel screenshot <url>` |
| Generate a PDF | `steel pdf <url>` |
| Multi-step interaction, login, forms, JS-heavy pages | `steel browser` session |
| Anti-bot / CAPTCHA sites | `steel browser --stealth` session |

**Start with `steel scrape` when you only need page content.** Escalate to `steel browser` when the page requires interaction or JavaScript rendering.

## API tools (one-shot, no session needed)

```bash
# Scrape — returns Markdown by default (use --json flag for structured output)
steel scrape https://example.com
steel scrape https://example.com --format html
steel scrape https://example.com --use-proxy

# Screenshot
steel screenshot https://example.com
steel screenshot https://example.com --full-page

# PDF
steel pdf https://example.com
```

## Interactive browser session

### Core workflow

1. **Start** a named session
2. **Navigate** to the target URL
3. **Snapshot** to get page state and element refs
4. **Interact** using `@eN` refs from the snapshot
5. **Re-snapshot** after every navigation or DOM change (refs expire)
6. **Stop** the session when done

```bash
steel browser start --session my-task --session-timeout 3600000
steel browser navigate https://example.com --session my-task
steel browser snapshot -i --session my-task
steel browser fill @e3 "search term" --session my-task
steel browser click @e7 --session my-task
steel browser wait --load networkidle --session my-task
steel browser snapshot -i --session my-task
steel browser stop --session my-task
```

**Rules:**
- Always use the same `--session <name>` on every command.
- Never use an `@eN` ref without a fresh snapshot — refs expire after navigation or DOM changes.
- Prefer element refs from `snapshot -i` over CSS selectors. Use `-c` for large DOMs, `-d 3` to limit depth.
- Use `batch` to combine multiple commands into a single invocation for efficiency.

### Batch execution

Run multiple commands in one CLI call. Each quoted string is one command.

```bash
# Navigate and snapshot in one call
steel browser batch "navigate https://example.com" "snapshot -i" --session my-task

# Action + re-snapshot (no separate snapshot call needed)
steel browser batch "click @e3" "snapshot -i" --session my-task

# Multiple actions without intermediate snapshots
steel browser batch "fill @e1 Seoul" "fill @e2 Tokyo" "click @e5" --session my-task

# Stop on first error with --bail
steel browser batch "click @e3" "snapshot -i" --session my-task --bail
```

Use `batch` when:
- You need to snapshot after an action (most common case)
- You are filling multiple form fields in sequence
- You want to reduce the number of CLI invocations

Use separate commands when you need to read the output of one command before deciding the next.

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
steel browser navigate <url> --session <name>
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

### Screenshots (in-session)

```bash
steel browser screenshot -o ./page.png --session <name>
steel browser screenshot --full --session <name>
steel browser screenshot --selector "#chart" --session <name>
```

Top-level `steel screenshot <url>` and `steel pdf <url>` are stateless one-shot API calls — they do not take `--session` or `-o` flags. Use `steel browser screenshot` for in-session captures.

### Cookies and storage

```bash
steel browser cookies --session <name>
steel browser cookies set <name> <value> --session <name>
steel browser cookies set <name> <value> --domain .example.com
steel browser cookies clear --session <name>
steel browser storage session --session <name>
steel browser storage session set authToken "abc123" --session <name>
```

### Browser settings

```bash
steel browser set viewport 1920 1080 --session <name>
steel browser set geo 37.7749 -122.4194 --session <name>
steel browser set offline on --session <name>
steel browser set useragent "Custom UA" --session <name>
```

### CAPTCHA

```bash
steel browser start --session <name> --stealth
steel browser captcha status --wait --session <name>
steel browser captcha solve --session <name>
```

### eval for capability gaps

Use `steel browser eval "<js>" --session <name>` when no direct command exists. Common uses: network interception, DOM state injection, complex form widgets.

## Commands that do NOT exist

Do not attempt these — they will fail.

| Does NOT exist | Use instead |
|---|---|
| `steel browser record` / `video` | `steel browser live` for viewer URL |
| `steel browser network` / `route` | `eval` with fetch monkey-patch |
| `steel browser console` / `errors` | `eval` with console interceptor |
| `steel browser frame` | `eval` with iframe contentDocument |
| `steel browser tabs` (plural) | `steel browser tab list` (singular) |
| `steel browser execute` / `run` | `steel browser eval` |
| `steel browser resize` | `steel browser set viewport W H` |
| `steel browser geolocation` | `steel browser set geo LAT LON` |
| `steel browser pdf` | Top-level `steel pdf <url>` |

## SDK integration (programmatic)

If you need to use Steel from code instead of the CLI:

### Python (Playwright)

```bash
pip install steel-sdk playwright
```

```python
from playwright.sync_api import sync_playwright
from steel import Steel
import os

client = Steel(steel_api_key=os.environ["STEEL_API_KEY"])
session = client.sessions.create()

try:
    pw = sync_playwright().start()
    browser = pw.chromium.connect_over_cdp(session.websocket_url)
    page = browser.contexts[0].pages[0]

    page.goto("https://example.com")
    print(page.title())
finally:
    browser.close()
    pw.stop()
    client.sessions.release(session.id)
```

### Node.js (Puppeteer)

```bash
npm install steel-sdk puppeteer
```

```typescript
import Steel from 'steel-sdk';
import puppeteer from 'puppeteer';

const client = new Steel({ steelAPIKey: process.env.STEEL_API_KEY });
const session = await client.sessions.create();

try {
  const browser = await puppeteer.connect({
    browserWSEndpoint: `${session.websocketUrl}?apiKey=${process.env.STEEL_API_KEY}`,
  });
  const page = await browser.newPage();
  await page.goto('https://example.com');
  console.log(await page.title());
  await browser.disconnect();
} finally {
  await client.sessions.release(session.id);
}
```

**Key pattern:** Create session → connect via CDP websocket URL → use any browser library → release session.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `steel: command not found` | `curl -sSf https://setup.steel.dev/install.sh \| sh && export PATH="$HOME/.steel/bin:$PATH"` |
| `Missing browser auth` | `steel login` or set `STEEL_API_KEY` env var |
| Authentication error in SDK | `export STEEL_API_KEY="your_key"` |
| `No running session` | Check session name; `steel browser stop --all` then restart |
| Stale element refs | Re-run `steel browser snapshot -i` before interacting |
| CAPTCHA blocking | `steel browser start --stealth --session <name>` |
| Session timeout | Add `--session-timeout 3600000` for tasks over 5 minutes |

If you run into issues, refer to https://docs.steel.dev or run `steel --help`.
