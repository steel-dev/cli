# Steel Browser Commands

Every `steel browser` action sends one request to the session's commands API (`POST /v1/sessions/{id}/commands`). The browser engine runs next to Chrome inside the session, so the CLI keeps no process running between commands.

## Sessions

```bash
steel browser start [--session <name>]   # create a session (default name: default)
steel browser sessions                   # list local sessions that are still live
steel browser live                       # print the live viewer URL
steel browser stop [--session <name>]    # release the session (-a for all)
steel browser close                      # same as stop (aliases: quit, exit)
```

- Actions without `--session` use the `default` session. It starts on first use.
- `--session <name>` targets a named session. A session ID also works, so you can drive a session that the SDK, the MCP server, or the dashboard created.
- New sessions are released after 10 minutes without commands. Change this with `steel browser start --inactivity-timeout <ms>`. `0` turns it off.
- The CLI keeps each session in `~/.config/steel/browser-sessions/<name>.json`: the session ID, the last operation id, the current tab, the event cursor, and the last snapshot.

## Navigation

```bash
steel browser open <url>         # aliases: navigate, goto. Adds https:// when the scheme is missing
steel browser open <url> --wait-until networkidle
steel browser back
steel browser forward
steel browser reload
```

`open` waits up to 15 seconds for the page to load.

## Snapshot and refs

```bash
steel browser snapshot           # full accessibility tree
steel browser snapshot -i        # interactive elements only (recommended)
steel browser snapshot -c        # lines with refs or values, and their ancestors
steel browser snapshot -d 3      # limit depth
```

Snapshot lines carry refs such as `[ref=e5]`. Pass a ref (`e5`, `@e5`) or a CSS selector wherever a command takes `<sel>`.

- A ref names one element. It fails with `stale_ref` when the element left the page or now shows different content. Take a new snapshot then.
- A ref knows its tab, so commands on refs work on any tab.

## Interaction

```bash
steel browser click <sel>              # --count 2 for a double-click
steel browser dblclick <sel>
steel browser fill <sel> <value>       # replace the value
steel browser type <sel> <text>        # type key by key; --clear, --delay <ms>
steel browser press <key>              # Enter, Tab, Escape, Control+a (alias: key)
steel browser keydown <key>            # hold a key; keyup releases it
steel browser keyup <key>
steel browser hover <sel>
steel browser focus <sel>
steel browser check <sel>
steel browser uncheck <sel>
steel browser select <sel> <value...>
steel browser clear <sel>
steel browser selectall <sel>
steel browser scroll [up|down|left|right] [px]   # -s <sel> scrolls an element
steel browser drag <source> <target>
steel browser upload <sel> <file...>   # local files are uploaded to the session first
steel browser mouse click <x> <y>      # move, down, up, click, wheel; --button, --clicks, --dx, --dy
```

Add `--diff` to any command to print what it changed on the page, compared with your last snapshot.

## Reading the page

```bash
steel browser get text <sel>       # visible text of an element
steel browser get html <sel>       # inner HTML of an element
steel browser get markdown [sel]   # main content of an element or the page as Markdown
steel browser get value <sel>
steel browser get attr <sel> <name>
steel browser get url
steel browser get title
steel browser get count <sel>
steel browser get box <sel>
steel browser is visible|enabled|checked <sel>
steel browser find <value>         # CSS by default; --by role|text|label|placeholder|alt|title|testid, --name, --exact
steel browser content              # page HTML
steel browser eval <script>        # run JavaScript and print its value
steel browser screenshot           # -o <path>, --full, --selector <sel>, --format, --quality
steel browser pdf -o page.pdf      # headless sessions only
```

## Waiting

```bash
steel browser wait 2000                         # fixed time in ms (at most 30000)
steel browser wait --selector <sel> [--state visible|hidden|attached|detached]
steel browser wait --text "Welcome"
steel browser wait --url checkout               # contains; use * for patterns
steel browser wait --load networkidle           # load, domcontentloaded, networkidle
steel browser wait --download [-o <path>]       # wait for a download and save it
```

`--timeout <ms>` sets the limit (default 30000, at most 50000).

## Tabs

```bash
steel browser tab list           # * marks the current tab
steel browser tab new [url]      # open a tab and switch to it
steel browser tab switch t2
steel browser tab close [t2]     # the current tab if omitted
```

Commands without a ref run on the current tab. When a page opens a tab, the CLI prints `Tab t3 opened from t1` and stays on the current tab.

## Dialogs and long commands

```bash
steel browser dialog accept [d-2]      # or dismiss; --text answers a prompt
steel browser status x-7               # wait for a command that is still running
steel browser cancel x-7               # stop it
```

Dialogs stay open until you answer them. A command that opens one prints the dialog id.

## Batch

```bash
steel browser batch "fill e3 Seoul" "fill e4 Tokyo" "click e5" "snapshot -i"
```

Batch sends all commands in one request (at most 50). They run in order and stop at the first failure. `tab new`, `tab switch`, `tab close`, `dialog`, `status`, `cancel`, and `close` must run on their own.

## Retries and output

- The CLI sends a random `Idempotency-Key` and the last operation id. If the API cannot reach the browser, it retries with the same key, so the command never runs twice.
- When a command's response is lost, run the same command again. You get the stored result instead of a second run, with the note `This is the stored result of an earlier run`.
- Text output prints the result. Notes about dialogs, new tabs, downloads, and replays go to stderr.
- `--json` prints the engine response under `data`, with any `notices`.
- A failed command exits with code 1 and prints the engine error code, such as `stale_ref`, `not_actionable`, or `tab_required`.

## Removed in 0.5

| Command | Instead |
|---|---|
| `cookies` | `eval "document.cookie"`; keep logins with `start --profile <name>` |
| `storage` | `eval "localStorage.getItem('key')"` |
| `set viewport/geo/offline/headers/useragent` | not supported on a running session |
| `highlight` | `screenshot --selector <sel>` |
| `bringtofront` | not needed; commands bring their tab to the front |
| `diff snapshot`, `diff screenshot` | `--diff` on any command |
| `scrollintoview` | `hover <sel>`, or `scroll --selector <container>` |
| `setvalue` | `fill` |
| `get styles` | `eval "getComputedStyle(...)"` |
| `click --new-tab`, `click --button` | `get attr <sel> href` + `tab new <url>`; `get box` + `mouse click` |
| `snapshot --selector`, `snapshot --urls` | `get text/html <sel>`; `get attr <ref> href` |
| `screenshot --annotate` | `snapshot -i` |
| `wait --function` | `wait --text`, `--selector`, or `--url` |
| `navigate --header` | not supported |

These commands still parse in 0.5 and print the alternative.
