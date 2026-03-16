# Steel Browser Commands Reference

Use this reference when you need command-level patterns for navigation, interaction, extraction, and debugging.

For any command, run `steel browser <command> --help` for full flag details.

## Navigation

```bash
steel browser open <url>
steel browser navigate <url> --wait-until networkidle
steel browser navigate <url> --header "Authorization: Bearer ..."
steel browser back
steel browser forward
steel browser reload
steel browser close
```

`open` is an alias for `navigate`.

## Snapshot and element discovery

```bash
steel browser snapshot                      # all elements (default)
steel browser snapshot -i                   # interactive elements only
steel browser snapshot -c                   # compact output
steel browser snapshot -d 3                 # limit tree depth
steel browser snapshot -s "#main"           # scope to subtree
steel browser snapshot -C                   # include cursor position
steel browser snapshot -i -c -d 3           # combine short flags
```

Snapshot returns an accessibility tree with element refs (`@e1`, `@e2`, ...).
Use these refs in subsequent commands instead of CSS selectors.

## Interactions with element refs

```bash
steel browser click @e1
steel browser click @e1 --new-tab
steel browser click @e1 --button right
steel browser click @e1 --count 2
steel browser dblclick @e1
steel browser focus @e1
steel browser fill @e2 "text"
steel browser type @e2 "text"
steel browser type @e2 "text" --clear
steel browser type @e2 "text" --delay 50
steel browser press Enter
steel browser press Control+a
steel browser hover @e1
steel browser check @e1
steel browser uncheck @e1
steel browser select @e1 "value"
steel browser select @e1 "a" "b"
steel browser clear @e1
steel browser selectall @e1
steel browser scroll down 500
steel browser scroll up
steel browser scroll down 200 --selector ".panel"
steel browser scrollintoview @e1
steel browser setvalue @e1 "raw value"
```

`setvalue` sets the DOM value property directly without triggering input events.
Use `fill` or `type` for normal form interaction.

## Information extraction

```bash
steel browser get text @e1
steel browser get html @e1
steel browser get value @e1
steel browser get attr @e1 href
steel browser get title
steel browser get url
steel browser get count ".item"
steel browser get box @e1
steel browser get styles @e1
steel browser get styles @e1 --property color --property font-size
```

There is no top-level `steel browser extract` command.
Use `steel browser get ...`, `steel browser snapshot`, and `steel browser find ...`.

## State checks

```bash
steel browser is visible @e1
steel browser is enabled @e1
steel browser is checked @e1
```

## Find elements

```bash
steel browser find ".item"
steel browser find "a[href*=login]"
```

Returns a list of matching elements with index, tag name, text, and visibility.

## Waiting and synchronization

```bash
steel browser wait -t "Success"
steel browser wait --selector ".loaded"
steel browser wait --selector ".modal" --state hidden
steel browser wait -u "/dashboard"
steel browser wait --fn "window.ready"
steel browser wait --load networkidle
steel browser wait --timeout 5000 -t "Done"
```

Short flags: `-t` text, `-u` url, `-f` function, `-l` load state.
Long aliases: `--fn` = `--function`, `--load` = `--load-state`.
All wait conditions default to 30 second timeout. Override with `--timeout <ms>`.

## Screenshots

```bash
steel browser screenshot
steel browser screenshot -o ./page.png
steel browser screenshot --full
steel browser screenshot --selector "#chart"
steel browser screenshot --annotate
steel browser screenshot --format jpeg --quality 80
```

## Page content

```bash
steel browser content
steel browser eval "document.title"
steel browser eval "document.querySelectorAll('a').length"
```

## Tabs

```bash
steel browser tab list
steel browser tab new
steel browser tab new https://example.com
steel browser tab switch 2
steel browser tab close
steel browser tab close 2
```

## Window

```bash
steel browser bringtofront
```

## Session lifecycle

```bash
steel browser start
steel browser start --session <name>
steel browser sessions
steel browser live
steel browser stop
steel browser stop --all
```

## CAPTCHA control

```bash
steel browser start --session <name> --stealth
steel browser captcha status
steel browser captcha status --wait
steel browser captcha status --wait --timeout 120000
steel browser captcha solve --session <name>
```

## Quick help

```bash
steel browser --help
steel browser <command> --help
steel browser get --help
steel browser is --help
steel browser tab --help
```
