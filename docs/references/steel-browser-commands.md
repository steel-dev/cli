# Steel Browser Commands (Synced + Transformed)

> Generated file. Do not edit manually.
> Source: https://raw.githubusercontent.com/vercel-labs/agent-browser/b59dc4c82c0583b60849d110f27982fac85f1a07/skills/agent-browser/references/commands.md
> Pinned commit: `b59dc4c82c0583b60849d110f27982fac85f1a07`
> Generated at: 2026-02-26T00:37:52.138Z

## Notes

- Upstream `agent-browser` command-prefix usage is transformed to `steel browser` for migration-friendly examples.
- Runtime-specific env vars and config names (for example `AGENT_BROWSER_*`) are preserved as upstream runtime details.
- Steel-native lifecycle commands (`steel browser start|stop|sessions|live`) are documented in `./steel-browser.md`.

# Command Reference (Steel Browser Adaptation)

Complete reference for all steel browser commands. For quick start and common patterns, see SKILL.md.

## Navigation

```bash
steel browser open <url>      # Navigate to URL (aliases: goto, navigate)
                              # Supports: https://, http://, file://, about:, data://
                              # Auto-prepends https:// if no protocol given
steel browser back            # Go back
steel browser forward         # Go forward
steel browser reload          # Reload page
steel browser close           # Close browser (aliases: quit, exit)
steel browser connect 9222    # Connect to browser via CDP port
```

## Snapshot (page analysis)

```bash
steel browser snapshot            # Full accessibility tree
steel browser snapshot -i         # Interactive elements only (recommended)
steel browser snapshot -c         # Compact output
steel browser snapshot -d 3       # Limit depth to 3
steel browser snapshot -s "#main" # Scope to CSS selector
```

## Interactions (use @refs from snapshot)

```bash
steel browser click @e1           # Click
steel browser click @e1 --new-tab # Click and open in new tab
steel browser dblclick @e1        # Double-click
steel browser focus @e1           # Focus element
steel browser fill @e2 "text"     # Clear and type
steel browser type @e2 "text"     # Type without clearing
steel browser press Enter         # Press key (alias: key)
steel browser press Control+a     # Key combination
steel browser keydown Shift       # Hold key down
steel browser keyup Shift         # Release key
steel browser hover @e1           # Hover
steel browser check @e1           # Check checkbox
steel browser uncheck @e1         # Uncheck checkbox
steel browser select @e1 "value"  # Select dropdown option
steel browser select @e1 "a" "b"  # Select multiple options
steel browser scroll down 500     # Scroll page (default: down 300px)
steel browser scrollintoview @e1  # Scroll element into view (alias: scrollinto)
steel browser drag @e1 @e2        # Drag and drop
steel browser upload @e1 file.pdf # Upload files
```

## Get Information

```bash
steel browser get text @e1        # Get element text
steel browser get html @e1        # Get innerHTML
steel browser get value @e1       # Get input value
steel browser get attr @e1 href   # Get attribute
steel browser get title           # Get page title
steel browser get url             # Get current URL
steel browser get count ".item"   # Count matching elements
steel browser get box @e1         # Get bounding box
steel browser get styles @e1      # Get computed styles (font, color, bg, etc.)
```

## Check State

```bash
steel browser is visible @e1      # Check if visible
steel browser is enabled @e1      # Check if enabled
steel browser is checked @e1      # Check if checked
```

## Screenshots and PDF

```bash
steel browser screenshot          # Save to temporary directory
steel browser screenshot path.png # Save to specific path
steel browser screenshot --full   # Full page
steel browser pdf output.pdf      # Save as PDF
```

## Video Recording

```bash
steel browser record start ./demo.webm    # Start recording
steel browser click @e1                   # Perform actions
steel browser record stop                 # Stop and save video
steel browser record restart ./take2.webm # Stop current + start new
```

## Wait

```bash
steel browser wait @e1                     # Wait for element
steel browser wait 2000                    # Wait milliseconds
steel browser wait --text "Success"        # Wait for text (or -t)
steel browser wait --url "**/dashboard"    # Wait for URL pattern (or -u)
steel browser wait --load networkidle      # Wait for network idle (or -l)
steel browser wait --fn "window.ready"     # Wait for JS condition (or -f)
```

## Mouse Control

```bash
steel browser mouse move 100 200      # Move mouse
steel browser mouse down left         # Press button
steel browser mouse up left           # Release button
steel browser mouse wheel 100         # Scroll wheel
```

## Semantic Locators (alternative to refs)

```bash
steel browser find role button click --name "Submit"
steel browser find text "Sign In" click
steel browser find text "Sign In" click --exact      # Exact match only
steel browser find label "Email" fill "user@test.com"
steel browser find placeholder "Search" type "query"
steel browser find alt "Logo" click
steel browser find title "Close" click
steel browser find testid "submit-btn" click
steel browser find first ".item" click
steel browser find last ".item" click
steel browser find nth 2 "a" hover
```

## Browser Settings

```bash
steel browser set viewport 1920 1080          # Set viewport size
steel browser set device "iPhone 14"          # Emulate device
steel browser set geo 37.7749 -122.4194       # Set geolocation (alias: geolocation)
steel browser set offline on                  # Toggle offline mode
steel browser set headers '{"X-Key":"v"}'     # Extra HTTP headers
steel browser set credentials user pass       # HTTP basic auth (alias: auth)
steel browser set media dark                  # Emulate color scheme
steel browser set media light reduced-motion  # Light mode + reduced motion
```

## Cookies and Storage

```bash
steel browser cookies                     # Get all cookies
steel browser cookies set name value      # Set cookie
steel browser cookies clear               # Clear cookies
steel browser storage local               # Get all localStorage
steel browser storage local key           # Get specific key
steel browser storage local set k v       # Set value
steel browser storage local clear         # Clear all
```

## Network

```bash
steel browser network route <url>              # Intercept requests
steel browser network route <url> --abort      # Block requests
steel browser network route <url> --body '{}'  # Mock response
steel browser network unroute [url]            # Remove routes
steel browser network requests                 # View tracked requests
steel browser network requests --filter api    # Filter requests
```

## Tabs and Windows

```bash
steel browser tab                 # List tabs
steel browser tab new [url]       # New tab
steel browser tab 2               # Switch to tab by index
steel browser tab close           # Close current tab
steel browser tab close 2         # Close tab by index
steel browser window new          # New window
```

## Frames

```bash
steel browser frame "#iframe"     # Switch to iframe
steel browser frame main          # Back to main frame
```

## Dialogs

```bash
steel browser dialog accept [text]  # Accept dialog
steel browser dialog dismiss        # Dismiss dialog
```

## JavaScript

```bash
steel browser eval "document.title"          # Simple expressions only
steel browser eval -b "<base64>"             # Any JavaScript (base64 encoded)
steel browser eval --stdin                   # Read script from stdin
```

Use `-b`/`--base64` or `--stdin` for reliable execution. Shell escaping with nested quotes and special characters is error-prone.

```bash
# Base64 encode your script, then:
steel browser eval -b "ZG9jdW1lbnQucXVlcnlTZWxlY3RvcignW3NyYyo9Il9uZXh0Il0nKQ=="

# Or use stdin with heredoc for multiline scripts:
cat <<'EOF' | steel browser eval --stdin
const links = document.querySelectorAll('a');
Array.from(links).map(a => a.href);
EOF
```

## State Management

```bash
steel browser state save auth.json    # Save cookies, storage, auth state
steel browser state load auth.json    # Restore saved state
```

## Global Options

```bash
steel browser --session <name> ...    # Isolated browser session
steel browser --json ...              # JSON output for parsing
steel browser --headed ...            # Show browser window (not headless)
steel browser --full ...              # Full page screenshot (-f)
steel browser --cdp <port> ...        # Connect via Chrome DevTools Protocol
steel browser -p <provider> ...       # Cloud browser provider (--provider)
steel browser --proxy <url> ...       # Use proxy server
steel browser --proxy-bypass <hosts>  # Hosts to bypass proxy
steel browser --headers <json> ...    # HTTP headers scoped to URL's origin
steel browser --executable-path <p>   # Custom browser executable
steel browser --extension <path> ...  # Load browser extension (repeatable)
steel browser --ignore-https-errors   # Ignore SSL certificate errors
steel browser --help                  # Show help (-h)
steel browser --version               # Show version (-V)
steel browser <command> --help        # Show detailed help for a command
```

## Debugging

```bash
steel browser --headed open example.com   # Show browser window
steel browser --cdp 9222 snapshot         # Connect via CDP port
steel browser connect 9222                # Alternative: connect command
steel browser console                     # View console messages
steel browser console --clear             # Clear console
steel browser errors                      # View page errors
steel browser errors --clear              # Clear errors
steel browser highlight @e1               # Highlight element
steel browser trace start                 # Start recording trace
steel browser trace stop trace.zip        # Stop and save trace
steel browser profiler start              # Start Chrome DevTools profiling
steel browser profiler stop trace.json    # Stop and save profile
```

## Environment Variables

```bash
AGENT_BROWSER_SESSION="mysession"            # Default session name
AGENT_BROWSER_EXECUTABLE_PATH="/path/chrome" # Custom browser path
AGENT_BROWSER_EXTENSIONS="/ext1,/ext2"       # Comma-separated extension paths
AGENT_BROWSER_PROVIDER="browserbase"         # Cloud browser provider
AGENT_BROWSER_STREAM_PORT="9223"             # WebSocket streaming port
AGENT_BROWSER_HOME="/path/to/agent-browser"  # Custom install location
```
