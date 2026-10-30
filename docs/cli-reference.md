# Steel CLI Reference

Steel CLI - browser automation for AI agents. This file is generated from `steel describe --all` and API metadata; regenerate it with `npm run docs:generate`.

## Table of Contents

- [steel scrape](#steel-scrape)
- [steel screenshot](#steel-screenshot)
- [steel pdf](#steel-pdf)
- [steel browser](#steel-browser)
- [steel browser start](#steel-browser-start)
- [steel browser stop](#steel-browser-stop)
- [steel browser sessions](#steel-browser-sessions)
- [steel browser live](#steel-browser-live)
- [steel browser captcha](#steel-browser-captcha)
- [steel browser captcha solve](#steel-browser-captcha-solve)
- [steel browser captcha status](#steel-browser-captcha-status)
- [steel browser batch](#steel-browser-batch)
- [steel browser navigate](#steel-browser-navigate)
- [steel browser back](#steel-browser-back)
- [steel browser forward](#steel-browser-forward)
- [steel browser reload](#steel-browser-reload)
- [steel browser click](#steel-browser-click)
- [steel browser dblclick](#steel-browser-dblclick)
- [steel browser fill](#steel-browser-fill)
- [steel browser type](#steel-browser-type)
- [steel browser press](#steel-browser-press)
- [steel browser keydown](#steel-browser-keydown)
- [steel browser keyup](#steel-browser-keyup)
- [steel browser hover](#steel-browser-hover)
- [steel browser focus](#steel-browser-focus)
- [steel browser check](#steel-browser-check)
- [steel browser uncheck](#steel-browser-uncheck)
- [steel browser select](#steel-browser-select)
- [steel browser clear](#steel-browser-clear)
- [steel browser selectall](#steel-browser-selectall)
- [steel browser scroll](#steel-browser-scroll)
- [steel browser drag](#steel-browser-drag)
- [steel browser upload](#steel-browser-upload)
- [steel browser mouse](#steel-browser-mouse)
- [steel browser snapshot](#steel-browser-snapshot)
- [steel browser screenshot](#steel-browser-screenshot)
- [steel browser pdf](#steel-browser-pdf)
- [steel browser eval](#steel-browser-eval)
- [steel browser find](#steel-browser-find)
- [steel browser content](#steel-browser-content)
- [steel browser get](#steel-browser-get)
- [steel browser get text](#steel-browser-get-text)
- [steel browser get html](#steel-browser-get-html)
- [steel browser get markdown](#steel-browser-get-markdown)
- [steel browser get value](#steel-browser-get-value)
- [steel browser get attr](#steel-browser-get-attr)
- [steel browser get url](#steel-browser-get-url)
- [steel browser get title](#steel-browser-get-title)
- [steel browser get count](#steel-browser-get-count)
- [steel browser get box](#steel-browser-get-box)
- [steel browser is](#steel-browser-is)
- [steel browser is visible](#steel-browser-is-visible)
- [steel browser is enabled](#steel-browser-is-enabled)
- [steel browser is checked](#steel-browser-is-checked)
- [steel browser wait](#steel-browser-wait)
- [steel browser tab](#steel-browser-tab)
- [steel browser tab list](#steel-browser-tab-list)
- [steel browser tab new](#steel-browser-tab-new)
- [steel browser tab switch](#steel-browser-tab-switch)
- [steel browser tab close](#steel-browser-tab-close)
- [steel browser dialog](#steel-browser-dialog)
- [steel browser status](#steel-browser-status)
- [steel browser cancel](#steel-browser-cancel)
- [steel browser close](#steel-browser-close)
- [steel sessions](#steel-sessions)
- [steel sessions list](#steel-sessions-list)
- [steel sessions get](#steel-sessions-get)
- [steel sessions release](#steel-sessions-release)
- [steel sessions logs](#steel-sessions-logs)
- [steel sessions agent-logs](#steel-sessions-agent-logs)
- [steel sessions traces](#steel-sessions-traces)
- [steel init](#steel-init)
- [steel login](#steel-login)
- [steel logout](#steel-logout)
- [steel credentials](#steel-credentials)
- [steel credentials list](#steel-credentials-list)
- [steel credentials create](#steel-credentials-create)
- [steel credentials update](#steel-credentials-update)
- [steel credentials delete](#steel-credentials-delete)
- [steel dev](#steel-dev)
- [steel dev install](#steel-dev-install)
- [steel dev start](#steel-dev-start)
- [steel dev stop](#steel-dev-stop)
- [steel forge](#steel-forge)
- [steel config](#steel-config)
- [steel update](#steel-update)
- [steel cache](#steel-cache)
- [steel profile](#steel-profile)
- [steel profile list](#steel-profile-list)
- [steel profile import](#steel-profile-import)
- [steel profile sync](#steel-profile-sync)
- [steel profile delete](#steel-profile-delete)
- [steel describe](#steel-describe)
- [steel doctor](#steel-doctor)
- [steel skills](#steel-skills)
- [steel skills list](#steel-skills-list)
- [steel skills install](#steel-skills-install)
- [steel skills update](#steel-skills-update)
- [steel skills doctor](#steel-skills-doctor)
- [steel skills open](#steel-skills-open)
- [steel skills paths](#steel-skills-paths)
- [steel completion](#steel-completion)

## Global Options

These options are accepted by every command.

- `--json` (boolean, optional): Output all results as JSON (structured output for automation)
- `--local` (boolean, optional): Use local Steel runtime instead of cloud
- `--api-url` (string, optional): Steel API endpoint URL (overrides --local and STEEL_API_URL)
- `-h, --help`: Print help for a command
- `-V, --version`: Print the Steel CLI version

## steel scrape

Scrape webpage content

### Usage

```bash
steel scrape
```

### Parameters

- `url` (string, optional): Target URL to scrape
- `--format` (string, optional): Comma-separated output formats: html, readability, cleaned_html, markdown
- `-d, --delay` (string, optional): Delay before scraping in milliseconds
- `--pdf` (boolean, optional): Include a generated PDF in the response
- `--screenshot` (boolean, optional): Include a generated screenshot in the response
- `--use-proxy` (boolean, optional): Use a Steel-managed residential proxy
- `-r, --region` (string, optional): Region identifier for request execution

## steel screenshot

Capture a screenshot of a webpage

### Usage

```bash
steel screenshot
```

### Parameters

- `url` (string, optional): Target URL to capture
- `-d, --delay` (string, optional): Delay before capturing in milliseconds
- `-f, --full-page` (boolean, optional): Capture a full-page screenshot
- `--use-proxy` (boolean, optional): Use a Steel-managed residential proxy
- `-r, --region` (string, optional): Region identifier for request execution

## steel pdf

Generate a PDF from a webpage

### Usage

```bash
steel pdf
```

### Parameters

- `url` (string, optional): Target URL to generate PDF from
- `-d, --delay` (string, optional): Delay before generating in milliseconds
- `--use-proxy` (boolean, optional): Use a Steel-managed residential proxy
- `-r, --region` (string, optional): Region identifier for request execution

## steel browser

Browser session management and automation

### Usage

```bash
steel browser
```

### Subcommands

- `start`: Create a browser session
- `stop`: Stop a browser session
- `sessions`: List active browser sessions
- `live`: Open the live session viewer
- `captcha`: CAPTCHA management
- `batch`: Run several browser commands in one request
- `navigate`: Navigate to a URL
- `back`: Navigate back in history
- `forward`: Navigate forward in history
- `reload`: Reload the current page
- `click`: Click an element
- `dblclick`: Double-click an element
- `fill`: Fill an input field (replaces the current value)
- `type`: Type text into an element key by key (appends to the current value)
- `press`: Press a key or chord (Enter, Escape, Tab, Control+a)
- `keydown`: Hold a key down until keyup
- `keyup`: Release a key held with keydown
- `hover`: Hover over an element
- `focus`: Focus an element
- `check`: Check a checkbox, radio button, or switch
- `uncheck`: Uncheck a checkbox or switch
- `select`: Select option(s) in a <select> element
- `clear`: Clear an input field
- `selectall`: Select all text in an input
- `scroll`: Scroll the page or an element
- `drag`: Drag an element onto another
- `upload`: Upload files to a file input
- `mouse`: Mouse input at a point, for canvases and maps
- `snapshot`: Take an accessibility tree snapshot with element refs
- `screenshot`: Take a screenshot
- `pdf`: Save the page as a PDF (headless sessions only)
- `eval`: Run JavaScript in the page and print its value
- `find`: Find elements and print their refs
- `content`: Get the page HTML
- `get`: Get information about elements or the page
- `is`: Check element state
- `wait`: Wait for an element, text, URL, load state, download, or a fixed time
- `tab`: List, open, switch, or close tabs
- `dialog`: Accept or dismiss an open dialog
- `status`: Wait for a command that is still running
- `cancel`: Stop a command that is still running
- `close`: Close the browser session

## steel browser start

Create a browser session

### Usage

```bash
steel browser start
```

### Parameters

- `--stealth` (boolean, optional): Enable stealth mode (humanize interactions + auto CAPTCHA)
- `-p, --proxy` (string, optional): Use a residential proxy
- `--session-timeout` (string, optional): Session timeout in milliseconds (create-time only)
- `--inactivity-timeout` (string, optional): Inactivity timeout in milliseconds: release the session when no command, CDP message, or remote input arrives for this long. Defaults to 600000 (10 minutes); pass 0 to disable (create-time only)
- `--session-solve-captcha` (boolean, optional): Enable manual CAPTCHA solving on new sessions (create-time only)
- `--profile` (string, optional): Named profile to persist browser state across sessions
- `--update-profile` (boolean, optional): Save session state back to the profile when the session ends
- `--namespace` (string, optional): Credential namespace
- `--credentials` (boolean, optional): Inject credentials

## steel browser stop

Stop a browser session

### Usage

```bash
steel browser stop
```

### Parameters

- `-a, --all` (boolean, optional): Stop all sessions

## steel browser sessions

List active browser sessions

### Usage

```bash
steel browser sessions
```

## steel browser live

Open the live session viewer

### Usage

```bash
steel browser live
```

## steel browser captcha

CAPTCHA management

### Usage

```bash
steel browser captcha
```

### Subcommands

- `solve`: Solve a CAPTCHA on the current page
- `status`: Check CAPTCHA status

## steel browser captcha solve

Solve a CAPTCHA on the current page

### Usage

```bash
steel browser captcha solve
```

### Parameters

- `--session-id` (string, optional): Explicit session ID (overrides --session name lookup)
- `--page-id` (string, optional): Page ID
- `--url` (string, optional): Page URL for targeted CAPTCHA solving
- `--task-id` (string, optional): CAPTCHA task ID for targeted solving

## steel browser captcha status

Check CAPTCHA status

### Usage

```bash
steel browser captcha status
```

### Parameters

- `--session-id` (string, optional): Explicit session ID (overrides --session name lookup)
- `--page-id` (string, optional): Page ID
- `-w, --wait` (boolean, optional): Wait for terminal status
- `--timeout` (string, optional): Timeout in milliseconds for --wait mode
- `--interval` (string, optional): Poll interval in milliseconds for --wait mode

## steel browser batch

Run several browser commands in one request

### Usage

```bash
steel browser batch
```

### Parameters

- `commands` (string[], required): Commands to run in order (each as a quoted string, e.g. "click e3"). They stop at the first failure

## steel browser navigate

Navigate to a URL

### Usage

```bash
steel browser navigate
```

Aliases: `open`, `goto`

### Parameters

- `url` (string, required): URL to navigate to
- `--wait-until` (enum, optional): Wait condition: load, domcontentloaded, networkidle

## steel browser back

Navigate back in history

### Usage

```bash
steel browser back
```

## steel browser forward

Navigate forward in history

### Usage

```bash
steel browser forward
```

## steel browser reload

Reload the current page

### Usage

```bash
steel browser reload
```

## steel browser click

Click an element

### Usage

```bash
steel browser click
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector
- `--count` (string, optional): Number of clicks (2 for double-click)

## steel browser dblclick

Double-click an element

### Usage

```bash
steel browser dblclick
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser fill

Fill an input field (replaces the current value)

### Usage

```bash
steel browser fill
```

### Parameters

- `selector` (string, required): Element ref or CSS selector
- `value` (string[], optional): Value to fill

## steel browser type

Type text into an element key by key (appends to the current value)

### Usage

```bash
steel browser type
```

### Parameters

- `selector` (string, required): Element ref or CSS selector
- `text` (string[], optional): Text to type
- `--clear` (boolean, optional): Clear the field before typing
- `--delay` (string, optional): Delay between keystrokes in milliseconds (at most 1000)

## steel browser press

Press a key or chord (Enter, Escape, Tab, Control+a)

### Usage

```bash
steel browser press
```

Aliases: `key`

### Parameters

- `key` (string, required): Key to press (e.g. Enter, Escape, Tab, Control+a)

## steel browser keydown

Hold a key down until keyup

### Usage

```bash
steel browser keydown
```

### Parameters

- `key` (string, required): Key name (e.g. Shift, Control, ArrowDown)

## steel browser keyup

Release a key held with keydown

### Usage

```bash
steel browser keyup
```

### Parameters

- `key` (string, required): Key name (e.g. Shift, Control, ArrowDown)

## steel browser hover

Hover over an element

### Usage

```bash
steel browser hover
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser focus

Focus an element

### Usage

```bash
steel browser focus
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser check

Check a checkbox, radio button, or switch

### Usage

```bash
steel browser check
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser uncheck

Uncheck a checkbox or switch

### Usage

```bash
steel browser uncheck
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser select

Select option(s) in a <select> element

### Usage

```bash
steel browser select
```

### Parameters

- `selector` (string, required): Select element ref or CSS selector
- `values` (string[], optional): Option value(s) or label(s) to select

## steel browser clear

Clear an input field

### Usage

```bash
steel browser clear
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser selectall

Select all text in an input

### Usage

```bash
steel browser selectall
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser scroll

Scroll the page or an element

### Usage

```bash
steel browser scroll
```

### Parameters

- `direction` (enum, optional): Direction: up, down, left, right (default: down)
- `amount` (string, optional): Scroll amount in pixels (default: 300)
- `-s, --selector` (string, optional): Element to scroll instead of the page

## steel browser drag

Drag an element onto another

### Usage

```bash
steel browser drag
```

### Parameters

- `source` (string, required): Element to drag (ref or CSS selector)
- `target` (string, required): Element to drop on (ref or CSS selector)

## steel browser upload

Upload files to a file input

### Usage

```bash
steel browser upload
```

### Parameters

- `selector` (string, required): File input ref or CSS selector
- `files` (string[], optional): Local files to upload, or names of files already in the session

## steel browser mouse

Mouse input at a point, for canvases and maps

### Usage

```bash
steel browser mouse
```

### Parameters

- `action` (enum, required): move, down, up, click, or wheel
- `x` (string, optional): X in CSS pixels of the visible page (needed for move, click, and wheel)
- `y` (string, optional): Y in CSS pixels of the visible page (needed for move, click, and wheel)
- `--button` (enum, optional): Mouse button: left, right, middle
- `--clicks` (string, optional): Click count for click (1 to 3)
- `--dx` (string, optional): Horizontal wheel delta
- `--dy` (string, optional): Vertical wheel delta

## steel browser snapshot

Take an accessibility tree snapshot with element refs

### Usage

```bash
steel browser snapshot
```

### Parameters

- `-i, --interactive` (boolean, optional): Show only interactive elements
- `-c, --compact` (boolean, optional): Keep only lines with refs or values, and their ancestors
- `-d, --max-depth` (string, optional): Maximum nesting depth

## steel browser screenshot

Take a screenshot

### Usage

```bash
steel browser screenshot
```

### Parameters

- `-f, --full-page` (boolean, optional): Capture the full scrollable page
- `-o, --output` (string, optional): Output file path
- `--selector` (string, optional): Capture one element (ref or CSS selector)
- `--format` (enum, optional): Image format: png, jpeg, webp (default: from the output extension)
- `--quality` (string, optional): JPEG/WebP quality (0-100)

## steel browser pdf

Save the page as a PDF (headless sessions only)

### Usage

```bash
steel browser pdf
```

### Parameters

- `-o, --output` (string, optional): Output file path
- `--landscape` (boolean, optional): Use landscape orientation
- `--print-background` (boolean, optional): Print background graphics

## steel browser eval

Run JavaScript in the page and print its value

### Usage

```bash
steel browser eval
```

### Parameters

- `script` (string, required): JavaScript expression or function body; its value is printed

## steel browser find

Find elements and print their refs

### Usage

```bash
steel browser find
```

### Parameters

- `value` (string, required): What to look for (a CSS selector unless --by says otherwise)
- `--by` (enum, optional): Locator kind
- `--name` (string, optional): Accessible name, with --by role
- `--exact` (boolean, optional): Match the text exactly instead of as a case-insensitive substring

## steel browser content

Get the page HTML

### Usage

```bash
steel browser content
```

## steel browser get

Get information about elements or the page

### Usage

```bash
steel browser get
```

### Subcommands

- `text`: Get the visible text of an element
- `html`: Get the inner HTML of an element
- `markdown`: Get the main content of an element or the page as Markdown
- `value`: Get an input or textarea value
- `attr`: Get an element attribute
- `url`: Get the page URL
- `title`: Get the page title
- `count`: Count matching elements
- `box`: Get an element's bounding box

## steel browser get text

Get the visible text of an element

### Usage

```bash
steel browser get text
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser get html

Get the inner HTML of an element

### Usage

```bash
steel browser get html
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser get markdown

Get the main content of an element or the page as Markdown

### Usage

```bash
steel browser get markdown
```

### Parameters

- `selector` (string, optional): Element ref (e.g. e5) or CSS selector; the whole page if omitted

## steel browser get value

Get an input or textarea value

### Usage

```bash
steel browser get value
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser get attr

Get an element attribute

### Usage

```bash
steel browser get attr
```

### Parameters

- `selector` (string, required): Element ref or CSS selector
- `attribute` (string, required): Attribute name

## steel browser get url

Get the page URL

### Usage

```bash
steel browser get url
```

## steel browser get title

Get the page title

### Usage

```bash
steel browser get title
```

## steel browser get count

Count matching elements

### Usage

```bash
steel browser get count
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser get box

Get an element's bounding box

### Usage

```bash
steel browser get box
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser is

Check element state

### Usage

```bash
steel browser is
```

### Subcommands

- `visible`: Check if an element is visible
- `enabled`: Check if an element is enabled
- `checked`: Check if an element is checked

## steel browser is visible

Check if an element is visible

### Usage

```bash
steel browser is visible
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser is enabled

Check if an element is enabled

### Usage

```bash
steel browser is enabled
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser is checked

Check if an element is checked

### Usage

```bash
steel browser is checked
```

### Parameters

- `selector` (string, required): Element ref (e.g. e5) or CSS selector

## steel browser wait

Wait for an element, text, URL, load state, download, or a fixed time

### Usage

```bash
steel browser wait
```

### Parameters

- `ms` (string, optional): Fixed time to wait in milliseconds (at most 30000)
- `--timeout` (string, optional): Timeout in milliseconds (at most 50000)
- `-t, --text` (string, optional): Wait for text to appear on the page
- `--selector` (string, optional): Wait for an element (ref or CSS selector)
- `--state` (enum, optional): Element state: visible, hidden, attached, detached
- `-u, --url` (string, optional): Wait for the URL to contain this string (`*` matches anything)
- `-l, --load-state` (enum, optional): Wait for a load state: load, domcontentloaded, networkidle
- `--download` (boolean, optional): Wait for a download to finish and save it
- `-o, --output` (string, optional): Where to save the download (default: its file name in the current directory)

## steel browser tab

List, open, switch, or close tabs

### Usage

```bash
steel browser tab
```

### Subcommands

- `list`: List open tabs
- `new`: Open a new tab and switch to it
- `switch`: Switch to a tab
- `close`: Close a tab (the current tab if omitted)

## steel browser tab list

List open tabs

### Usage

```bash
steel browser tab list
```

## steel browser tab new

Open a new tab and switch to it

### Usage

```bash
steel browser tab new
```

### Parameters

- `url` (string, optional): URL to open (defaults to about:blank)

## steel browser tab switch

Switch to a tab

### Usage

```bash
steel browser tab switch
```

### Parameters

- `tab` (string, required): Tab id from `tab list`, such as t2

## steel browser tab close

Close a tab (the current tab if omitted)

### Usage

```bash
steel browser tab close
```

### Parameters

- `tab` (string, optional): Tab id from `tab list`, such as t2

## steel browser dialog

Accept or dismiss an open dialog

### Usage

```bash
steel browser dialog
```

### Parameters

- `action` (enum, required): accept or dismiss
- `id` (string, optional): Dialog id such as d-2 (default: the last dialog the CLI saw)
- `--text` (string, optional): Answer for a prompt dialog

## steel browser status

Wait for a command that is still running

### Usage

```bash
steel browser status
```

### Parameters

- `execution` (string, required): Execution id such as x-7

## steel browser cancel

Stop a command that is still running

### Usage

```bash
steel browser cancel
```

### Parameters

- `execution` (string, required): Execution id such as x-7

## steel browser close

Close the browser session

### Usage

```bash
steel browser close
```

Aliases: `quit`, `exit`

## steel sessions

Cloud session management and debugging

### Usage

```bash
steel sessions
```

### Subcommands

- `list`: List cloud sessions
- `get`: Get one cloud session
- `release`: Release a cloud session
- `logs`: Read browser event logs for a session
- `agent-logs`: Read raw CDP-derived agent events for a session
- `traces`: Read semantic agent trace timeline for a session

## steel sessions list

List cloud sessions

### Usage

```bash
steel sessions list
```

### Parameters

- `--limit` (string, optional): Number of sessions to return
- `--cursor-id` (string, optional): Cursor ID for pagination
- `--status` (string, optional): Filter by status: live, released, or failed

### API Operations

- `get_sessions` (implemented): `GET /v1/sessions`
  Example: `steel sessions list --status live --limit 20`

## steel sessions get

Get one cloud session

### Usage

```bash
steel sessions get
```

### Parameters

- `session_id` (string, optional): Cloud session ID
- `--session` (string, optional): Resolve session ID from a local browser session name

### API Operations

- `get_session` (implemented): `GET /v1/sessions/{id}`
  Example: `steel sessions get <session-id>`

## steel sessions release

Release a cloud session

### Usage

```bash
steel sessions release
```

### Parameters

- `session_id` (string, optional): Cloud session ID
- `-a, --all` (boolean, optional): Release all active cloud sessions
- `--session` (string, optional): Resolve session ID from a local browser session name

### API Operations

- `release_session` (implemented): `POST /v1/sessions/{id}/release`
  Example: `steel sessions release <session-id>`
- `release_all_sessions` (implemented): `POST /v1/sessions/release`
  Example: `steel sessions release --all`

## steel sessions logs

Read browser event logs for a session

### Usage

```bash
steel sessions logs
```

### Parameters

- `session_id` (string, optional): Cloud session ID
- `--session` (string, optional): Resolve session ID from a local browser session name
- `-f, --follow` (boolean, optional): Stream live frames over WebSocket after printing historical data
- `--namespace` (string, optional): Filter by namespace
- `--start-time` (string, optional): ISO timestamp lower bound
- `--since` (string, optional): Alias for --start-time
- `--end-time` (string, optional): ISO timestamp upper bound
- `--event-type` (string[], optional): Event type filter, repeatable or comma-separated
- `--limit` (string, optional): Number of events to return
- `--offset` (string, optional): Pagination offset

### API Operations

- `get_session_logs` (implemented): `GET /v1/sessions/{id}/logs`
  Example: `steel sessions logs <session-id> --follow`
  Streaming: websocket `/v1/sessions/{id}/logs`

## steel sessions agent-logs

Read raw CDP-derived agent events for a session

### Usage

```bash
steel sessions agent-logs
```

### Parameters

- `session_id` (string, optional): Cloud session ID
- `--session` (string, optional): Resolve session ID from a local browser session name
- `-f, --follow` (boolean, optional): Stream live frames over WebSocket after printing historical data
- `--namespace` (string, optional): Filter by namespace
- `--start-time` (string, optional): ISO timestamp lower bound
- `--since` (string, optional): Alias for --start-time
- `--end-time` (string, optional): ISO timestamp upper bound
- `--event-type` (string[], optional): Event type filter, repeatable or comma-separated
- `--limit` (string, optional): Number of events to return
- `--offset` (string, optional): Pagination offset

### API Operations

- `get_session_agent_logs` (implemented): `GET /v1/sessions/{id}/agent-logs`
  Example: `steel sessions agent-logs <session-id> --limit 100`

## steel sessions traces

Read semantic agent trace timeline for a session

### Usage

```bash
steel sessions traces
```

### Parameters

- `session_id` (string, optional): Cloud session ID
- `--session` (string, optional): Resolve session ID from a local browser session name
- `-f, --follow` (boolean, optional): Stream live trace frames over WebSocket after printing historical data
- `--namespace` (string, optional): Filter by namespace
- `--start-time` (string, optional): ISO timestamp lower bound
- `--since` (string, optional): Alias for --start-time
- `--end-time` (string, optional): ISO timestamp upper bound
- `--event-type` (string[], optional): Event type filter, repeatable or comma-separated

### API Operations

- `get_session_agent_traces` (implemented): `GET /v1/sessions/{id}/agent-traces`
  Example: `steel sessions traces <session-id> --follow`
  Streaming: websocket `/v1/sessions/{id}/agent-traces`

## steel init

One-command onboarding: login + verify + install agent skills

### Usage

```bash
steel init
```

### Parameters

- `--agent` (boolean, optional): Run in agent mode: auto-accept interactive prompts and print agent-friendly output. Designed for AI coding agents
- `--skills` (string[], optional): Open the Steel skills installer flow. With no value, lets you choose skills interactively
- `--no-skills` (boolean, optional): Skip Steel skill installation

## steel login

Login to Steel CLI

### Usage

```bash
steel login
```

Aliases: `auth`

## steel logout

Logout from Steel CLI

### Usage

```bash
steel logout
```

## steel credentials

Manage stored credentials

### Usage

```bash
steel credentials
```

### Subcommands

- `list`: List stored credentials
- `create`: Create a new credential
- `update`: Update an existing credential
- `delete`: Delete a credential

## steel credentials list

List stored credentials

### Usage

```bash
steel credentials list
```

### Parameters

- `-n, --namespace` (string, optional): Filter by namespace
- `--origin` (string, optional): Filter by origin URL

## steel credentials create

Create a new credential

### Usage

```bash
steel credentials create
```

### Parameters

- `--origin` (string, optional): Origin URL to associate the credential with
- `-u, --username` (string, optional): Username
- `-p, --password` (string, optional): Password
- `--totp-secret` (string, optional): TOTP secret for two-factor authentication
- `-n, --namespace` (string, optional): Credential namespace
- `--label` (string, optional): Human-readable label

## steel credentials update

Update an existing credential

### Usage

```bash
steel credentials update
```

### Parameters

- `--origin` (string, optional): Origin URL of the credential to update
- `-u, --username` (string, optional): New username
- `-p, --password` (string, optional): New password
- `--totp-secret` (string, optional): New TOTP secret
- `-n, --namespace` (string, optional): Credential namespace
- `--label` (string, optional): New human-readable label

## steel credentials delete

Delete a credential

### Usage

```bash
steel credentials delete
```

### Parameters

- `--origin` (string, optional): Origin URL of the credential to delete
- `-n, --namespace` (string, optional): Credential namespace

## steel dev

Local development runtime

### Usage

```bash
steel dev
```

### Subcommands

- `install`: Install the local Steel Browser runtime
- `start`: Start the local runtime containers
- `stop`: Stop the local runtime containers

## steel dev install

Install the local Steel Browser runtime

### Usage

```bash
steel dev install
```

### Parameters

- `--repo-url` (string, optional): Git repository URL for local Steel Browser runtime
- `-V, --verbose` (boolean, optional): Enable verbose git command output

## steel dev start

Start the local runtime containers

### Usage

```bash
steel dev start
```

### Parameters

- `-p, --port` (string, optional): API port for local Steel Browser runtime
- `-V, --verbose` (boolean, optional): Enable verbose Docker command output
- `-d, --docker-check` (boolean, optional): Only verify Docker availability and exit

## steel dev stop

Stop the local runtime containers

### Usage

```bash
steel dev stop
```

### Parameters

- `-V, --verbose` (boolean, optional): Enable verbose Docker command output

## steel forge

Scaffold a new project from a template

### Usage

```bash
steel forge
```

### Parameters

- `template` (string, optional): Template to start from
- `-n, --name` (string, optional): Project name
- `--list` (boolean, optional): List available templates and exit (respects --json)

## steel config

Show current configuration

### Usage

```bash
steel config
```

## steel update

Update to the latest version

### Usage

```bash
steel update
```

### Parameters

- `-f, --force` (boolean, optional): Force update even if already on latest version
- `-c, --check` (boolean, optional): Only check for updates without installing

## steel cache

Manage Steel CLI cache

### Usage

```bash
steel cache
```

### Parameters

- `-c, --clean` (boolean, optional): Remove all cached files and directories

## steel profile

Manage named Steel browser profiles

### Usage

```bash
steel profile
```

### Subcommands

- `list`: List all saved Steel browser profiles
- `import`: Import a local browser profile into Steel
- `sync`: Sync a local browser profile to an existing Steel profile
- `delete`: Delete a saved Steel browser profile

## steel profile list

List all saved Steel browser profiles

### Usage

```bash
steel profile list
```

## steel profile import

Import a local browser profile into Steel

### Usage

```bash
steel profile import
```

### Parameters

- `name` (string, optional): Steel profile name to save as
- `--from` (string, optional): Browser profile directory to import from (e.g. "Default", "Profile 1")
- `--browser` (string, optional): Browser to import from (chrome, edge, brave, arc, opera, vivaldi)
- `--full` (boolean, optional): Include all profile data (IndexedDB, History, Bookmarks, etc.)

## steel profile sync

Sync a local browser profile to an existing Steel profile

### Usage

```bash
steel profile sync
```

### Parameters

- `name` (string, optional): Steel profile name to sync
- `--from` (string, optional): Browser profile directory to sync from (overrides stored source)
- `--browser` (string, optional): Browser to sync from (overrides stored browser)
- `--full` (boolean, optional): Include all profile data (IndexedDB, History, Bookmarks, etc.)

## steel profile delete

Delete a saved Steel browser profile

### Usage

```bash
steel profile delete
```

### Parameters

- `name` (string, optional): Name of the profile to delete

## steel describe

Describe commands and parameters

### Usage

```bash
steel describe
```

### Parameters

- `path` (string[], optional): Command path to describe (e.g. "browser click")
- `--all` (boolean, optional): Dump the entire command tree (recursive)

## steel doctor

Check environment, auth, and connectivity

### Usage

```bash
steel doctor
```

### Parameters

- `--preflight` (boolean, optional): Only check auth and API connectivity

## steel skills

Manage Steel agent skills

### Usage

```bash
steel skills
```

### Subcommands

- `list`: List available Steel skills
- `install`: Install all or selected Steel skills through npx skills
- `update`: Update one or more installed Steel skills through npx skills
- `doctor`: Check Steel skills installation health
- `open`: Open an installed skill or docs page
- `paths`: Show detected agent install paths for Steel skills

## steel skills list

List available Steel skills

### Usage

```bash
steel skills list
```

### Parameters

- `--offline` (boolean, optional): Use cached or bundled manifest instead of fetching GitHub

## steel skills install

Install all or selected Steel skills through npx skills

### Usage

```bash
steel skills install
```

### Parameters

- `names` (string[], optional): Skill name(s) to install
- `--all` (boolean, optional): Install all Steel skills from the catalog
- `-a, --agent` (string, optional): Target agent passed to npx skills (-a)
- `-g, --global` (boolean, optional): Install globally through npx skills (-g)
- `-y, --yes` (boolean, optional): Non-interactive yes passed to npx skills (-y)

## steel skills update

Update one or more installed Steel skills through npx skills

### Usage

```bash
steel skills update
```

### Parameters

- `names` (string[], optional): Skill name(s) to update; all known Steel skills when omitted
- `-y, --yes` (boolean, optional): Non-interactive yes passed to npx skills (-y)

## steel skills doctor

Check Steel skills installation health

### Usage

```bash
steel skills doctor
```

### Parameters

- `--offline` (boolean, optional): Use cached or bundled manifest instead of fetching GitHub

## steel skills open

Open an installed skill or docs page

### Usage

```bash
steel skills open
```

### Parameters

- `name` (string, required): Skill name

## steel skills paths

Show detected agent install paths for Steel skills

### Usage

```bash
steel skills paths
```

## steel completion

Generate a shell completion script

### Usage

```bash
steel completion
```

### Parameters

- `shell` (enum, required): Shell to generate completions for
