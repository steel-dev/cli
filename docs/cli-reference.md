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
- [steel browser hover](#steel-browser-hover)
- [steel browser focus](#steel-browser-focus)
- [steel browser check](#steel-browser-check)
- [steel browser uncheck](#steel-browser-uncheck)
- [steel browser select](#steel-browser-select)
- [steel browser clear](#steel-browser-clear)
- [steel browser selectall](#steel-browser-selectall)
- [steel browser scroll](#steel-browser-scroll)
- [steel browser scrollintoview](#steel-browser-scrollintoview)
- [steel browser setvalue](#steel-browser-setvalue)
- [steel browser snapshot](#steel-browser-snapshot)
- [steel browser screenshot](#steel-browser-screenshot)
- [steel browser eval](#steel-browser-eval)
- [steel browser find](#steel-browser-find)
- [steel browser content](#steel-browser-content)
- [steel browser get](#steel-browser-get)
- [steel browser get text](#steel-browser-get-text)
- [steel browser get html](#steel-browser-get-html)
- [steel browser get value](#steel-browser-get-value)
- [steel browser get attr](#steel-browser-get-attr)
- [steel browser get url](#steel-browser-get-url)
- [steel browser get title](#steel-browser-get-title)
- [steel browser get count](#steel-browser-get-count)
- [steel browser get box](#steel-browser-get-box)
- [steel browser get styles](#steel-browser-get-styles)
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
- [steel browser cookies](#steel-browser-cookies)
- [steel browser cookies set](#steel-browser-cookies-set)
- [steel browser cookies clear](#steel-browser-cookies-clear)
- [steel browser storage](#steel-browser-storage)
- [steel browser storage local](#steel-browser-storage-local)
- [steel browser storage local set](#steel-browser-storage-local-set)
- [steel browser storage local clear](#steel-browser-storage-local-clear)
- [steel browser storage session](#steel-browser-storage-session)
- [steel browser storage session set](#steel-browser-storage-session-set)
- [steel browser storage session clear](#steel-browser-storage-session-clear)
- [steel browser drag](#steel-browser-drag)
- [steel browser upload](#steel-browser-upload)
- [steel browser highlight](#steel-browser-highlight)
- [steel browser set](#steel-browser-set)
- [steel browser set viewport](#steel-browser-set-viewport)
- [steel browser set geo](#steel-browser-set-geo)
- [steel browser set offline](#steel-browser-set-offline)
- [steel browser set headers](#steel-browser-set-headers)
- [steel browser set useragent](#steel-browser-set-useragent)
- [steel browser bringtofront](#steel-browser-bringtofront)
- [steel browser diff](#steel-browser-diff)
- [steel browser diff snapshot](#steel-browser-diff-snapshot)
- [steel browser diff screenshot](#steel-browser-diff-screenshot)
- [steel browser close](#steel-browser-close)
- [steel sessions](#steel-sessions)
- [steel sessions list](#steel-sessions-list)
- [steel sessions get](#steel-sessions-get)
- [steel sessions release](#steel-sessions-release)
- [steel sessions logs](#steel-sessions-logs)
- [steel sessions agent-logs](#steel-sessions-agent-logs)
- [steel sessions traces](#steel-sessions-traces)
- [steel computer](#steel-computer)
- [steel computer create](#steel-computer-create)
- [steel computer list](#steel-computer-list)
- [steel computer get](#steel-computer-get)
- [steel computer delete](#steel-computer-delete)
- [steel computer pause](#steel-computer-pause)
- [steel computer resume](#steel-computer-resume)
- [steel computer use](#steel-computer-use)
- [steel computer exec](#steel-computer-exec)
- [steel computer ssh](#steel-computer-ssh)
- [steel computer checkpoint](#steel-computer-checkpoint)
- [steel computer quota](#steel-computer-quota)
- [steel checkpoint](#steel-checkpoint)
- [steel checkpoint list](#steel-checkpoint-list)
- [steel checkpoint get](#steel-checkpoint-get)
- [steel checkpoint delete](#steel-checkpoint-delete)
- [steel checkpoint restore](#steel-checkpoint-restore)
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
- `--api-url` (string, optional): Explicit self-hosted API endpoint URL
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

- `start`: Create or attach to a browser session
- `stop`: Stop a browser session
- `sessions`: List active browser sessions
- `live`: Open the live session viewer
- `captcha`: CAPTCHA management
- `batch`: Run multiple browser commands in a single invocation
- `navigate`: Navigate to a URL
- `back`: Navigate back in history
- `forward`: Navigate forward in history
- `reload`: Reload the current page
- `click`: Click an element
- `dblclick`: Double-click an element
- `fill`: Fill an input field (clears existing value first)
- `type`: Type text into an element (appends to existing value)
- `press`: Press a keyboard key
- `hover`: Hover over an element
- `focus`: Focus an element
- `check`: Check a checkbox or radio button
- `uncheck`: Uncheck a checkbox
- `select`: Select option(s) from a dropdown
- `clear`: Clear an input field
- `selectall`: Select all text in an input
- `scroll`: Scroll the page or an element
- `scrollintoview`: Scroll an element into view
- `setvalue`: Set the value of an input element (without triggering events)
- `snapshot`: Take an accessibility tree snapshot
- `screenshot`: Take a screenshot
- `eval`: Evaluate JavaScript in the page
- `find`: Find all elements matching a selector
- `content`: Get the page HTML content
- `get`: Get information about elements or page
- `is`: Check element state
- `wait`: Wait for a condition (text, selector, URL, function, or timeout)
- `tab`: Manage tabs
- `cookies`: Manage browser cookies
- `storage`: Manage browser storage (localStorage/sessionStorage)
- `drag`: Drag and drop from one element to another
- `upload`: Upload files to a file input element
- `highlight`: Visually highlight an element
- `set`: Configure browser settings (viewport, geolocation, etc.)
- `bringtofront`: Bring the browser window to the foreground
- `diff`: Compare snapshots or screenshots
- `close`: Close the browser session

## steel browser start

Create or attach to a browser session

### Usage

```bash
steel browser start
```

### Parameters

- `--stealth` (boolean, optional): Enable stealth mode (humanize interactions + auto CAPTCHA)
- `-p, --proxy` (string, optional): Use a residential proxy
- `--session-timeout` (string, optional): Session timeout in milliseconds (create-time only)
- `--inactivity-timeout` (string, optional): Inactivity timeout in milliseconds: release the session when no CDP command or remote input is received for this long. Defaults to 120000 (2 minutes); pass 0 to disable (create-time only)
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

Run multiple browser commands in a single invocation

### Usage

```bash
steel browser batch
```

### Parameters

- `commands` (string[], required): Commands to execute (each as a quoted string, e.g. "click @e3")
- `--bail` (boolean, optional): Stop on first error

## steel browser navigate

Navigate to a URL

### Usage

```bash
steel browser navigate
```

Aliases: `open`, `goto`

### Parameters

- `url` (string, required): URL to navigate to
- `--wait-until` (string, optional): Wait condition: load, domcontentloaded, networkidle
- `--header` (string[], optional): Set request header (repeatable, format: "Key: Value")

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

- `selector` (string, required): Element selector or ref (e.g. @e1)
- `--button` (string, optional): Mouse button: left, right, middle
- `--count` (string, optional): Number of clicks (2 for double-click)
- `--new-tab` (boolean, optional): Open the link in a new tab instead of clicking

## steel browser dblclick

Double-click an element

### Usage

```bash
steel browser dblclick
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser fill

Fill an input field (clears existing value first)

### Usage

```bash
steel browser fill
```

### Parameters

- `selector` (string, required): Element selector or ref
- `value` (string[], optional): Value to fill

## steel browser type

Type text into an element (appends to existing value)

### Usage

```bash
steel browser type
```

### Parameters

- `selector` (string, required): Element selector or ref
- `text` (string[], optional): Text to type
- `--clear` (boolean, optional): Clear the field before typing
- `--delay` (string, optional): Delay between keystrokes in milliseconds

## steel browser press

Press a keyboard key

### Usage

```bash
steel browser press
```

Aliases: `key`

### Parameters

- `key` (string, required): Key to press (e.g. Enter, Escape, Tab, Control+a)

## steel browser hover

Hover over an element

### Usage

```bash
steel browser hover
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser focus

Focus an element

### Usage

```bash
steel browser focus
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser check

Check a checkbox or radio button

### Usage

```bash
steel browser check
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser uncheck

Uncheck a checkbox

### Usage

```bash
steel browser uncheck
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser select

Select option(s) from a dropdown

### Usage

```bash
steel browser select
```

### Parameters

- `selector` (string, required): Select element selector or ref
- `values` (string[], optional): Option value(s) to select

## steel browser clear

Clear an input field

### Usage

```bash
steel browser clear
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser selectall

Select all text in an input

### Usage

```bash
steel browser selectall
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser scroll

Scroll the page or an element

### Usage

```bash
steel browser scroll
```

### Parameters

- `direction` (string, optional): Direction: up, down, left, right
- `amount` (string, optional): Scroll amount in pixels (default: 300)
- `-s, --selector` (string, optional): Element selector to scroll

## steel browser scrollintoview

Scroll an element into view

### Usage

```bash
steel browser scrollintoview
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser setvalue

Set the value of an input element (without triggering events)

### Usage

```bash
steel browser setvalue
```

### Parameters

- `selector` (string, required): Element selector or ref
- `value` (string[], optional): Value to set

## steel browser snapshot

Take an accessibility tree snapshot

### Usage

```bash
steel browser snapshot
```

### Parameters

- `-i, --interactive` (boolean, optional): Show only interactive elements
- `-s, --selector` (string, optional): Restrict snapshot to a subtree
- `-c, --compact` (boolean, optional): Use compact output format
- `-d, --max-depth` (string, optional): Maximum nesting depth
- `-u, --urls` (boolean, optional): Include URLs in the snapshot

## steel browser screenshot

Take a screenshot

### Usage

```bash
steel browser screenshot
```

### Parameters

- `-f, --full-page` (boolean, optional): Capture the full scrollable page
- `-o, --output` (string, optional): Output file path
- `--selector` (string, optional): Restrict screenshot to an element
- `--format` (string, optional): Image format: png, jpeg, webp
- `--quality` (string, optional): JPEG/WebP quality (0-100)
- `--annotate` (boolean, optional): Annotate interactive elements on the screenshot

## steel browser eval

Evaluate JavaScript in the page

### Usage

```bash
steel browser eval
```

### Parameters

- `script` (string, required): JavaScript expression to evaluate

## steel browser find

Find all elements matching a selector

### Usage

```bash
steel browser find
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser content

Get the page HTML content

### Usage

```bash
steel browser content
```

## steel browser get

Get information about elements or page

### Usage

```bash
steel browser get
```

### Subcommands

- `text`: Get element text content
- `html`: Get element inner HTML
- `value`: Get input/textarea value
- `attr`: Get element attribute value
- `url`: Get current page URL
- `title`: Get current page title
- `count`: Count matching elements
- `box`: Get element bounding box
- `styles`: Get element CSS styles

## steel browser get text

Get element text content

### Usage

```bash
steel browser get text
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser get html

Get element inner HTML

### Usage

```bash
steel browser get html
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser get value

Get input/textarea value

### Usage

```bash
steel browser get value
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser get attr

Get element attribute value

### Usage

```bash
steel browser get attr
```

### Parameters

- `selector` (string, required): Element selector or ref
- `attribute` (string, required): Attribute name

## steel browser get url

Get current page URL

### Usage

```bash
steel browser get url
```

## steel browser get title

Get current page title

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

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser get box

Get element bounding box

### Usage

```bash
steel browser get box
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser get styles

Get element CSS styles

### Usage

```bash
steel browser get styles
```

### Parameters

- `selector` (string, required): Element selector or ref
- `--property` (string[], optional): CSS property names to query (returns all computed styles if omitted)

## steel browser is

Check element state

### Usage

```bash
steel browser is
```

### Subcommands

- `visible`: Check if element is visible
- `enabled`: Check if element is enabled
- `checked`: Check if element is checked

## steel browser is visible

Check if element is visible

### Usage

```bash
steel browser is visible
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser is enabled

Check if element is enabled

### Usage

```bash
steel browser is enabled
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser is checked

Check if element is checked

### Usage

```bash
steel browser is checked
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser wait

Wait for a condition (text, selector, URL, function, or timeout)

### Usage

```bash
steel browser wait
```

### Parameters

- `--timeout` (string, optional): Timeout in milliseconds
- `-t, --text` (string, optional): Wait for text to appear on page
- `--selector` (string, optional): Wait for a CSS selector
- `--state` (string, optional): Selector state: visible, hidden, attached, detached
- `-u, --url` (string, optional): Wait for URL to contain this string
- `-f, --function` (string, optional): Wait for a JS function to return truthy
- `-l, --load-state` (string, optional): Wait for load state: load, domcontentloaded, networkidle

## steel browser tab

Manage tabs

### Usage

```bash
steel browser tab
```

### Subcommands

- `list`: List open tabs
- `new`: Open a new tab
- `switch`: Switch to a tab by index
- `close`: Close a tab (active tab if no index given)

## steel browser tab list

List open tabs

### Usage

```bash
steel browser tab list
```

## steel browser tab new

Open a new tab

### Usage

```bash
steel browser tab new
```

### Parameters

- `url` (string, optional): URL to open (defaults to about:blank)

## steel browser tab switch

Switch to a tab by index

### Usage

```bash
steel browser tab switch
```

### Parameters

- `index` (string, required): Tab index to switch to

## steel browser tab close

Close a tab (active tab if no index given)

### Usage

```bash
steel browser tab close
```

### Parameters

- `index` (string, optional): Tab index to close (closes active tab if omitted)

## steel browser cookies

Manage browser cookies

### Usage

```bash
steel browser cookies
```

### Subcommands

- `set`: Set a cookie
- `clear`: Clear all cookies

## steel browser cookies set

Set a cookie

### Usage

```bash
steel browser cookies set
```

### Parameters

- `name` (string, required): Cookie name
- `value` (string, required): Cookie value
- `--domain` (string, optional): Cookie domain
- `--path` (string, optional): Cookie path
- `--secure` (boolean, optional): Secure flag
- `--http-only` (boolean, optional): HttpOnly flag

## steel browser cookies clear

Clear all cookies

### Usage

```bash
steel browser cookies clear
```

## steel browser storage

Manage browser storage (localStorage/sessionStorage)

### Usage

```bash
steel browser storage
```

### Subcommands

- `local`: Manage localStorage
- `session`: Manage sessionStorage

## steel browser storage local

Manage localStorage

### Usage

```bash
steel browser storage local
```

### Subcommands

- `set`: Set a storage value
- `clear`: Clear all values

## steel browser storage local set

Set a storage value

### Usage

```bash
steel browser storage local set
```

### Parameters

- `key` (string, required): Key to set
- `value` (string, required): Value to set

## steel browser storage local clear

Clear all values

### Usage

```bash
steel browser storage local clear
```

## steel browser storage session

Manage sessionStorage

### Usage

```bash
steel browser storage session
```

### Subcommands

- `set`: Set a storage value
- `clear`: Clear all values

## steel browser storage session set

Set a storage value

### Usage

```bash
steel browser storage session set
```

### Parameters

- `key` (string, required): Key to set
- `value` (string, required): Value to set

## steel browser storage session clear

Clear all values

### Usage

```bash
steel browser storage session clear
```

## steel browser drag

Drag and drop from one element to another

### Usage

```bash
steel browser drag
```

### Parameters

- `source` (string, required): Source element selector or ref
- `target` (string, required): Target element selector or ref

## steel browser upload

Upload files to a file input element

### Usage

```bash
steel browser upload
```

### Parameters

- `selector` (string, required): File input element selector or ref
- `files` (string[], optional): File paths to upload

## steel browser highlight

Visually highlight an element

### Usage

```bash
steel browser highlight
```

### Parameters

- `selector` (string, required): Element selector or ref (e.g. @e1)

## steel browser set

Configure browser settings (viewport, geolocation, etc.)

### Usage

```bash
steel browser set
```

### Subcommands

- `viewport`: Set viewport size
- `geo`: Set geolocation
- `offline`: Toggle offline mode
- `headers`: Set extra HTTP headers (JSON string)
- `useragent`: Set browser user agent string

## steel browser set viewport

Set viewport size

### Usage

```bash
steel browser set viewport
```

### Parameters

- `width` (string, required): Viewport width in pixels
- `height` (string, required): Viewport height in pixels
- `--scale` (string, optional): Device scale factor
- `--mobile` (boolean, optional): Emulate mobile device

## steel browser set geo

Set geolocation

### Usage

```bash
steel browser set geo
```

Aliases: `geolocation`

### Parameters

- `latitude` (string, required): Latitude
- `longitude` (string, required): Longitude
- `--accuracy` (string, optional): Accuracy in meters

## steel browser set offline

Toggle offline mode

### Usage

```bash
steel browser set offline
```

### Parameters

- `state` (string, required): "on" to enable offline mode, "off" to disable

## steel browser set headers

Set extra HTTP headers (JSON string)

### Usage

```bash
steel browser set headers
```

### Parameters

- `json` (string, required): JSON string of headers (e.g. '{"X-Key":"value"}')

## steel browser set useragent

Set browser user agent string

### Usage

```bash
steel browser set useragent
```

Aliases: `ua`

### Parameters

- `user_agent` (string[], optional): User agent string

## steel browser bringtofront

Bring the browser window to the foreground

### Usage

```bash
steel browser bringtofront
```

## steel browser diff

Compare snapshots or screenshots

### Usage

```bash
steel browser diff
```

### Subcommands

- `snapshot`: Compare current snapshot against a baseline
- `screenshot`: Compare current screenshot against a baseline image

## steel browser diff snapshot

Compare current snapshot against a baseline

### Usage

```bash
steel browser diff snapshot
```

### Parameters

- `-b, --baseline` (string, optional): Baseline snapshot text or file path
- `-s, --selector` (string, optional): Restrict snapshot to a subtree
- `-c, --compact` (boolean, optional): Use compact output format
- `-d, --max-depth` (string, optional): Maximum nesting depth

## steel browser diff screenshot

Compare current screenshot against a baseline image

### Usage

```bash
steel browser diff screenshot
```

### Parameters

- `-b, --baseline` (string, required): Baseline image file path (required)
- `-t, --threshold` (string, optional): Color difference threshold (0.0–1.0)
- `-o, --output` (string, optional): Save diff image to this path
- `-s, --selector` (string, optional): Restrict screenshot to an element
- `--full-page` (boolean, optional): Capture the full scrollable page

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
- `--session` (string, optional): Resolve session ID from a local daemon session name

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
- `--session` (string, optional): Resolve session ID from a local daemon session name

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
- `--session` (string, optional): Resolve session ID from a local daemon session name
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
- `--session` (string, optional): Resolve session ID from a local daemon session name
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
- `--session` (string, optional): Resolve session ID from a local daemon session name
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

## steel computer

Cloud computers: create, run commands, ssh

### Usage

```bash
steel computer
```

### Subcommands

- `create`: Create a computer
- `list`: List computers
- `get`: Get one computer
- `delete`: Delete a computer
- `pause`: Pause a running computer
- `resume`: Resume a paused computer
- `use`: Remember a computer as the default for other commands
- `exec`: Run one command in a computer
- `ssh`: Open an SSH session to a computer
- `checkpoint`: Save a computer as a checkpoint
- `quota`: Show how many computers and checkpoints you can have

## steel computer create

Create a computer

### Usage

```bash
steel computer create
```

### Parameters

- `--template` (string, required): Template name
- `--region` (string, optional): Region, for example us-east
- `--vcpu` (string, optional): Number of vCPUs
- `--memory` (string, optional): Memory in MiB
- `--disk` (string, optional): Disk in MiB
- `--timeout` (string, optional): Stop the computer after this many seconds of running time
- `--auto-pause` (boolean, optional): Pause instead of stopping when the timeout is reached
- `--wait` (boolean, optional): Wait until the computer is running
- `--use` (boolean, optional): Make the new computer the default for other commands

## steel computer list

List computers

### Usage

```bash
steel computer list
```

## steel computer get

Get one computer

### Usage

```bash
steel computer get
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)

## steel computer delete

Delete a computer

### Usage

```bash
steel computer delete
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)

## steel computer pause

Pause a running computer

### Usage

```bash
steel computer pause
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)

## steel computer resume

Resume a paused computer

### Usage

```bash
steel computer resume
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
- `--wait` (boolean, optional): Wait until the computer is running

## steel computer use

Remember a computer as the default for other commands

### Usage

```bash
steel computer use
```

### Parameters

- `computer_id` (string, optional): Computer ID to remember
- `--clear` (boolean, optional): Forget the remembered computer

## steel computer exec

Run one command in a computer

### Usage

```bash
steel computer exec
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
- `-c, --command` (string, optional): Shell command string, run by /bin/sh -c
- `--cwd` (string, optional): Working directory inside the computer
- `--env` (string[], optional): Environment variable for the command, repeatable
- `--timeout` (string, optional): Kill the command after this many seconds (at most 3600)
- `argv` (string[], optional): Program and arguments, given after `--`

### API Operations

- `exec_computer` (implemented): `POST /v1/computers/{id}/exec`
  Example: `steel computer exec <computer-id> -- <command>`

## steel computer ssh

Open an SSH session to a computer

### Usage

```bash
steel computer ssh
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
- `command` (string[], optional): Run this command instead of opening a shell, given after `--`

### API Operations

- `attach_computer_ssh` (implemented): `GET /v1/computers/{id}/ssh`
  Example: `steel computer ssh <computer-id>`
  Streaming: websocket `/v1/computers/{id}/ssh`

## steel computer checkpoint

Save a computer as a checkpoint

### Usage

```bash
steel computer checkpoint
```

### Parameters

- `computer_id` (string, optional): Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
- `--name` (string, optional): Name for the checkpoint
- `--wait` (boolean, optional): Wait until the checkpoint is ready

## steel computer quota

Show how many computers and checkpoints you can have

### Usage

```bash
steel computer quota
```

## steel checkpoint

Computer checkpoints: list, restore, delete

### Usage

```bash
steel checkpoint
```

### Subcommands

- `list`: List checkpoints
- `get`: Get one checkpoint
- `delete`: Delete a checkpoint
- `restore`: Start a new computer from a checkpoint

## steel checkpoint list

List checkpoints

### Usage

```bash
steel checkpoint list
```

## steel checkpoint get

Get one checkpoint

### Usage

```bash
steel checkpoint get
```

### Parameters

- `checkpoint_id` (string, required): Checkpoint ID

## steel checkpoint delete

Delete a checkpoint

### Usage

```bash
steel checkpoint delete
```

### Parameters

- `checkpoint_id` (string, required): Checkpoint ID

## steel checkpoint restore

Start a new computer from a checkpoint

### Usage

```bash
steel checkpoint restore
```

### Parameters

- `checkpoint_id` (string, required): Checkpoint ID
- `--timeout` (string, optional): Stop the computer after this many seconds of running time
- `--auto-pause` (boolean, optional): Pause instead of stopping when the timeout is reached
- `--wait` (boolean, optional): Wait until the computer is running
- `--use` (boolean, optional): Make the new computer the default for other commands

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
