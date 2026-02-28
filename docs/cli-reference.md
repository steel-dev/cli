# Steel CLI Reference

This is an auto-generated reference for the Steel CLI. The Steel CLI helps you create, run, and manage browser automation projects in the cloud.

## Table of Contents

- [steel cache](#steel-cache)
- [steel config](#steel-config)
- [steel docs](#steel-docs)
- [steel forge](#steel-forge)
- [steel login](#steel-login)
- [steel logout](#steel-logout)
- [steel pdf](#steel-pdf)
- [steel run](#steel-run)
- [steel scrape](#steel-scrape)
- [steel screenshot](#steel-screenshot)
- [steel settings](#steel-settings)
- [steel star](#steel-star)
- [steel support](#steel-support)
- [steel update](#steel-update)
- [steel browser captcha solve](#steel-browser-captcha-solve)
- [steel browser live](#steel-browser-live)
- [steel browser sessions](#steel-browser-sessions)
- [steel browser start](#steel-browser-start)
- [steel browser stop](#steel-browser-stop)
- [steel dev install](#steel-dev-install)
- [steel dev start](#steel-dev-start)
- [steel dev stop](#steel-dev-stop)

## Global Options

These options are available for most commands:

- **-h, --help**: Display help for a command
- **-v, --version**: Display Steel CLI version

## steel cache

Manage Steel CLI cache which is used to store files for quickly running scripts

### Usage

```
steel cache [options]
```

### Options

- -c, **--clean**: Remove all cached files and directories

## steel config

Display information about the current session

### Usage

```
steel config
```

## steel docs

Navigates to Steel Docs

### Usage

```
steel docs
```

## steel forge

Start a new project using the Steel CLI

### Usage

```
steel forge [template] [options]
```

### Available Templates

| Alias | Label | Language | Description |
|-------|-------|----------|-------------|
| `steel-auth-context-starter` | Reusing Authentication Context | TS | Template for Reusing Authentication Context automation |
| `steel-browser-use-starter` | Using Steel with Browser Use | PY | Template for Using Steel with Browser Use automation |
| `steel-claude-computer-use-node-starter` | Using Steel with Claude's Computer Use | TS | Template for Using Steel with Claude's Computer Use automation |
| `steel-claude-computer-use-python-starter` | Using Steel with Claude's Computer Use | PY | Template for Using Steel with Claude's Computer Use automation |
| `steel-credentials-starter` | Using Steel Credentials with Playwright | TS | Template for Using Steel Credentials with Playwright automation |
| `steel-files-api-starter` | Using the Steel Files API | TS | Template for Using the Steel Files API automation |
| `steel-magnitude-starter` | Using Steel with Magnitude | TS | Template for Using Steel with Magnitude automation |
| `steel-oai-computer-use-node-starter` | Using Steel with OpenAI's Computer Use | TS | Template for Using Steel with OpenAI's Computer Use automation |
| `steel-oai-computer-use-python-starter` | Using Steel with OpenAI's Computer Use | PY | Template for Using Steel with OpenAI's Computer Use automation |
| `steel-playwright-python-starter` | Drive a Steel Session with Playwright | PY | Template for Drive a Steel Session with Playwright automation |
| `steel-playwright-starter-js` | Using Playwright with Steel | JS | Template for Using Playwright with Steel automation |
| `steel-playwright-starter` | Drive a Steel Session with Playwright | TS | Template for Drive a Steel Session with Playwright automation |
| `steel-puppeteer-starter-js` | Using Puppeteer with Steel | JS | Template for Using Puppeteer with Steel automation |
| `steel-puppeteer-starter` | Drive a Steel Session with Puppeteer | TS | Template for Drive a Steel Session with Puppeteer automation |
| `steel-selenium-starter` | Using Selenium with Steel | PY | Template for Using Selenium with Steel automation |
| `steel-stagehand-node-starter` | Using Steel with Stagehand | TS | Template for Using Steel with Stagehand automation |
| `steel-stagehand-python-starter` | Using Steel with Stagehand | PY | Template for Using Steel with Stagehand automation |

### Arguments

- **template** (optional): Example template to start

### Options

- -n, **--name**: Name of the project
- -a, **--api_url**: API URL for Steel API
- **--api_key**: API Key for Steel API
- **--openai_key**: API Key for OpenAI
- **--anthropic_key**: API Key for Anthropic
- **--skip_auth**: Skip authentication

## steel login

Login to Steel CLI

### Usage

```
steel login
```

## steel logout

Logout from Steel CLI

### Usage

```
steel logout
```

## steel pdf

Generate a webpage PDF through the Steel API

### Usage

```
steel pdf [url] [options]
```

### Arguments

- **url** (optional): Target URL to convert

### Options

- -u, **--url**: Target URL to convert
- -d, **--delay**: Delay before PDF generation in milliseconds
- **--use-proxy**: Use a Steel-managed residential proxy
- -r, **--region**: Region identifier for request execution
- -l, **--local**: Send request to local Steel runtime mode
- **--api-url**: Explicit self-hosted API endpoint URL

## steel run

Run a Steel Cookbook automation instantly from the CLI — no setup, no files.

### Usage

```
steel run [template] [options]
```

### Available Templates

| Alias | Label | Language | Description |
|-------|-------|----------|-------------|
| `steel-auth-context-starter` | Reusing Authentication Context | TS | Template for Reusing Authentication Context automation |
| `steel-browser-use-starter` | Using Steel with Browser Use | PY | Template for Using Steel with Browser Use automation |
| `steel-claude-computer-use-node-starter` | Using Steel with Claude's Computer Use | TS | Template for Using Steel with Claude's Computer Use automation |
| `steel-claude-computer-use-python-starter` | Using Steel with Claude's Computer Use | PY | Template for Using Steel with Claude's Computer Use automation |
| `steel-credentials-starter` | Using Steel Credentials with Playwright | TS | Template for Using Steel Credentials with Playwright automation |
| `steel-files-api-starter` | Using the Steel Files API | TS | Template for Using the Steel Files API automation |
| `steel-magnitude-starter` | Using Steel with Magnitude | TS | Template for Using Steel with Magnitude automation |
| `steel-oai-computer-use-node-starter` | Using Steel with OpenAI's Computer Use | TS | Template for Using Steel with OpenAI's Computer Use automation |
| `steel-oai-computer-use-python-starter` | Using Steel with OpenAI's Computer Use | PY | Template for Using Steel with OpenAI's Computer Use automation |
| `steel-playwright-python-starter` | Drive a Steel Session with Playwright | PY | Template for Drive a Steel Session with Playwright automation |
| `steel-playwright-starter-js` | Using Playwright with Steel | JS | Template for Using Playwright with Steel automation |
| `steel-playwright-starter` | Drive a Steel Session with Playwright | TS | Template for Drive a Steel Session with Playwright automation |
| `steel-puppeteer-starter-js` | Using Puppeteer with Steel | JS | Template for Using Puppeteer with Steel automation |
| `steel-puppeteer-starter` | Drive a Steel Session with Puppeteer | TS | Template for Drive a Steel Session with Puppeteer automation |
| `steel-selenium-starter` | Using Selenium with Steel | PY | Template for Using Selenium with Steel automation |
| `steel-stagehand-node-starter` | Using Steel with Stagehand | TS | Template for Using Steel with Stagehand automation |
| `steel-stagehand-python-starter` | Using Steel with Stagehand | PY | Template for Using Steel with Stagehand automation |

### Arguments

- **template** (optional): Example template to run

### Options

- -a, **--api_url**: API URL for Steel API
- -o, **--view**: Open live session viewer
- -t, **--task**: Task to run
- **--api_key**: API Key for Steel API
- **--openai_key**: API Key for OpenAI
- **--anthropic_key**: API Key for Anthropic
- **--gemini_key**: API Key for Gemini
- **--skip_auth**: Skip authentication
- -h, **--help**: Show help

## steel scrape

Scrape webpage content through the Steel API (markdown output by default)

### Usage

```
steel scrape [url] [options]
```

### Arguments

- **url** (optional): Target URL to scrape

### Options

- -u, **--url**: Target URL to scrape
- **--format**: Comma-separated output formats: html, readability, cleaned_html, markdown
- **--raw**: Print full JSON response payload
- -d, **--delay**: Delay before scraping in milliseconds
- **--pdf**: Include a generated PDF in the scrape response
- **--screenshot**: Include a generated screenshot in the scrape response
- **--use-proxy**: Use a Steel-managed residential proxy
- -r, **--region**: Region identifier for request execution
- -l, **--local**: Send request to local Steel runtime mode
- **--api-url**: Explicit self-hosted API endpoint URL

## steel screenshot

Capture a webpage screenshot through the Steel API

### Usage

```
steel screenshot [url] [options]
```

### Arguments

- **url** (optional): Target URL to capture

### Options

- -u, **--url**: Target URL to capture
- -d, **--delay**: Delay before capture in milliseconds
- -f, **--full-page**: Capture the full page (not only the viewport)
- **--use-proxy**: Use a Steel-managed residential proxy
- -r, **--region**: Region identifier for request execution
- -l, **--local**: Send request to local Steel runtime mode
- **--api-url**: Explicit self-hosted API endpoint URL

## steel settings

Display current settings

### Usage

```
steel settings
```

## steel star

Navigates to Steel Browser Repository

### Usage

```
steel star
```

## steel support

Navigates to Steel Discord Server

### Usage

```
steel support
```

## steel update

Update Steel CLI to the latest version

### Usage

```
steel update [options]
```

### Options

- -f, **--force**: Force update even if already on latest version
- -c, **--check**: Only check for updates without installing

## steel browser captcha solve

Manually trigger CAPTCHA solving for a Steel browser session

### Usage

```
steel browser captcha solve [options]
```

### Options

- **--session-id**: Explicit Steel session id to target
- -s, **--session**: Named session key to resolve from local state
- -l, **--local**: Resolve session and execute solve call in local mode
- **--api-url**: Explicit self-hosted API endpoint URL
- **--page-id**: Optional page ID for targeted CAPTCHA solving
- **--url**: Optional page URL for targeted CAPTCHA solving
- **--task-id**: Optional CAPTCHA task ID for targeted solving
- **--raw**: Print the full raw API payload

## steel browser live

Print active or named session live-view URL

### Usage

```
steel browser live [options]
```

### Options

- -s, **--session**: Named session key to resolve live URL for
- -l, **--local**: Resolve live URL from local active session
- **--api-url**: Explicit self-hosted API endpoint URL

## steel browser sessions

List browser sessions as JSON

### Usage

```
steel browser sessions [options]
```

### Options

- -l, **--local**: List sessions from local Steel runtime
- **--api-url**: Explicit self-hosted API endpoint URL
- **--raw**: Include full raw API payload for each session

## steel browser start

Create or attach a Steel browser session (cloud by default)

### Usage

```
steel browser start [options]
```

### Options

- -l, **--local**: Start or attach a local Steel browser session
- **--api-url**: Explicit self-hosted API endpoint URL
- -s, **--session**: Named session key for create-or-attach behavior
- **--stealth**: Apply stealth preset on new sessions (humanized interactions + auto CAPTCHA solving / solveCaptcha=true)
- -p, **--proxy**: Proxy URL for new sessions (for example, http://user:pass@host:port)
- **--session-timeout**: Session timeout in milliseconds (create-time only)
- **--session-headless**: Create new sessions in headless mode (create-time only)
- **--session-region**: Preferred session region (create-time only)
- **--session-solve-captcha**: Enable manual CAPTCHA solving on new sessions (create-time only; use `steel browser captcha solve`)

## steel browser stop

Stop the active or named Steel browser session

### Usage

```
steel browser stop [options]
```

### Options

- -a, **--all**: Stop all live sessions in the active mode
- -s, **--session**: Named session key to stop
- -l, **--local**: Stop sessions from local Steel runtime mode
- **--api-url**: Explicit self-hosted API endpoint URL

## steel dev install

Install local Steel Browser runtime assets without starting containers

### Usage

```
steel dev install [options]
```

### Options

- **--repo-url**: Git repository URL for local Steel Browser runtime
- -V, **--verbose**: Enable verbose git command output

## steel dev start

Start local Steel Browser runtime containers (requires `steel dev install`)

### Usage

```
steel dev start [options]
```

### Options

- -p, **--port**: API port for local Steel Browser runtime
- -V, **--verbose**: Enable verbose Docker command output
- -d, **--docker_check**: Only verify Docker availability and exit

## steel dev stop

Stop local Steel Browser runtime containers

### Usage

```
steel dev stop [options]
```

### Options

- -V, **--verbose**: Enable verbose Docker command output

---

*This documentation was auto-generated from the Steel CLI source code.*
