# Steel CLI Reference

This is an auto-generated reference for the Steel CLI. The Steel CLI helps you create, run, and manage browser automation projects in the cloud.

## Table of Contents

- [Steel CLI Reference](#steel-cli-reference)
	- [Table of Contents](#table-of-contents)
	- [Global Options](#global-options)
	- [steel cache](#steel-cache)
		- [Usage](#usage)
		- [Options](#options)
	- [steel docs](#steel-docs)
		- [Usage](#usage-1)
	- [steel forge](#steel-forge)
		- [Usage](#usage-2)
		- [Available Templates](#available-templates)
		- [Arguments](#arguments)
		- [Options](#options-1)
	- [steel info](#steel-info)
		- [Usage](#usage-3)
	- [steel login](#steel-login)
		- [Usage](#usage-4)
	- [steel logout](#steel-logout)
		- [Usage](#usage-5)
	- [steel run](#steel-run)
		- [Usage](#usage-6)
		- [Available Templates](#available-templates-1)
		- [Arguments](#arguments-1)
		- [Options](#options-2)
	- [steel star](#steel-star)
		- [Usage](#usage-7)
	- [steel support](#steel-support)
		- [Usage](#usage-8)
	- [steel browser start](#steel-browser-start)
		- [Usage](#usage-9)
		- [Options](#options-3)
	- [steel browser stop](#steel-browser-stop)
		- [Usage](#usage-10)

## Global Options

These options are available for most commands:

- **-h, --help**: Display help for a command
- **-v, --version**: Display Steel CLI version

## steel cache

Manage Steel CLI cache

### Usage

```
steel cache [options]
```

### Options

- -c, **--clean**: Remove all cached files and directories

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

| Alias           | Label                                              | Language | Description                                                                |
| --------------- | -------------------------------------------------- | -------- | -------------------------------------------------------------------------- |
| `auth`          | Reusing Authentication Context                     | TS       | Template for Reusing Authentication Context automation                     |
| `browser-use`   | Using Steel with Browser Use                       | PY       | Template for Using Steel with Browser Use automation                       |
| `claude-cua`    | Using Steel with Claude's Computer Use             | TS       | Template for Using Steel with Claude's Computer Use automation             |
| `claude-cua-py` | Using Steel with Claude's Computer Use             | PY       | Template for Using Steel with Claude's Computer Use automation             |
| `creds`         | Using Steel Credentials with Playwright            | TS       | Template for Using Steel Credentials with Playwright automation            |
| `files`         | Using the Steel Files API                          | TS       | Template for Using the Steel Files API automation                          |
| `magnitude`     | Using Steel with Magnitude                         | TS       | Template for Using Steel with Magnitude automation                         |
| `oai-cua`       | Using Steel with OpenAI's Computer Use             | TS       | Template for Using Steel with OpenAI's Computer Use automation             |
| `oai-cua-py`    | Using Steel with OpenAI's Computer Use             | PY       | Template for Using Steel with OpenAI's Computer Use automation             |
| `playwright-py` | Drive a Steel Session with Playwright              | PY       | Template for Drive a Steel Session with Playwright automation              |
| `playwright-js` | Using Playwright with Steel                        | JS       | Template for Using Playwright with Steel automation                        |
| `playwright`    | Drive a Steel Session with Playwright              | TS       | Template for Drive a Steel Session with Playwright automation              |
| `puppeteer-js`  | Using Puppeteer with Steel                         | JS       | Template for Using Puppeteer with Steel automation                         |
| `puppeteer`     | Drive a Steel Session with Puppeteer               | TS       | Template for Drive a Steel Session with Puppeteer automation               |
| `selenium`      | Using Selenium with Steel                          | PY       | Template for Using Selenium with Steel automation                          |
| `stagehand`     | Using Steel with Stagehand                         | TS       | Template for Using Steel with Stagehand automation                         |
| `stagehand-py`  | Using Steel with Stagehand                         | PY       | Template for Using Steel with Stagehand automation                         |

### Arguments

- **template** (optional): Example template to start

### Options

- -n, **--name**: Name of the project
- -a, **--api_url**: API URL for Steel API
- **--api_key**: API Key for Steel API
- **--openai_key**: API Key for OpenAI
- **--skip_auth**: Skip authentication

## steel info

Display information about the current session

### Usage

```
steel info
```

## steel login

Login to Steel CLI

### Usage

```
steel login
```

### Options

- **--token** (`-t`): JWT token (or full callback URL with jwt parameter) from login redirect.

You can also use `STEEL_LOGIN_TOKEN` environment variable when running in automation/non-interactive flows.

## steel logout

Logout from Steel CLI

### Usage

```
steel logout
```

## steel run

Run a Steel Cookbook automation instantly from the CLI — no setup, no files, just quick execution in a temporary cache.

### Usage

```
steel run [template] [options]
```

### Available Templates

| Alias           | Label                                              | Language | Description                                                                |
| --------------- | -------------------------------------------------- | -------- | -------------------------------------------------------------------------- |
| `playwright-js` | Playwright                                         | JS       | Template for Playwright automation                                         |
| `playwright`    | Playwright + TypeScript                            | TS       | Template for Playwright + TypeScript automation                            |
| `puppeteer-js`  | Puppeteer                                          | JS       | Template for Puppeteer automation                                          |
| `puppeteer`     | Puppeteer + TypeScript                             | TS       | Template for Puppeteer + TypeScript automation                             |
| `files`         | Playwright + Files API Starter in TypeScript       | TS       | Template for Playwright + Files API Starter in TypeScript automation       |
| `creds`         | Playwright + Credentials API Starter in TypeScript | TS       | Template for Playwright + Credentials API Starter in TypeScript automation |
| `oai-cua`       | Steel + OpenAI Computer Use + TypeScript           | TS       | Template for Steel + OpenAI Computer Use + TypeScript automation           |
| `magnitude`     | Steel + Magnitude                                  | TS       | Template for Steel + Magnitude automation                                  |
| `browser-use`   | (Python) Steel + Browser Use                       | PY       | Template for (Python) Steel + Browser Use automation                       |
| `oai-cua-py`    | (Python) Steel + OpenAI Computer Use               | PY       | Template for (Python) Steel + OpenAI Computer Use automation               |
| `playwright-py` | (Python) Steel + Playwright                        | PY       | Template for (Python) Steel + Playwright automation                        |
| `selenium`      | (Python) Steel + Selenium                          | PY       | Template for (Python) Steel + Selenium automation                          |

### Arguments

- **template** (optional): Example template to run

### Options

- -a, **--api_url**: API URL for Steel API
- -o, **--view**: Open live session viewer
- -t, **--task**: Task to run
- **--api_key**: API Key for Steel API
- **--openai_key**: API Key for OpenAI
- **--skip_auth**: Skip authentication
- -h, **--help**: Show help

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

## steel browser start

Starts the development environment

### Usage

```
steel browser start [options]
```

### Options

- -p, **--port**: Port number
- -v, **--verbose**: Enable verbose logging
- -d, **--docker_check**: Verify Docker is running

## steel browser stop

Stops the development server

### Usage

```
steel browser stop
```

---

_This documentation was auto-generated from the Steel CLI source code._
