# Steel — Agent Onboarding Guide

You are an AI coding agent setting up Steel for your user. Steel is a cloud
browser API purpose-built for agents: your user wants you to be able to
navigate real web pages, fill forms, take screenshots, and bypass bot
protection on their behalf.

Your job is to get Steel installed, authenticated, and verified — **without
asking the user to do more than one thing** (clicking an authentication link).

---

## Hard rules — read these first

1. If any step fails, run `steel doctor` and report the failing check to the
   user. Do not guess at fixes — `steel doctor` will tell you exactly what's
   wrong and how to repair it.
2. Do not run `steel forge`, `steel dev install`, or other scaffolding
   commands unless the user explicitly asks for them. This guide is only
   about **getting Steel ready to use**, not creating a starter project.
3. If `steel --version` is already available and `steel doctor --preflight`
   reports healthy, Steel is already set up — tell the user and stop. Do not
   re-run login.

---

## Steps

### Step 1 — Add the Steel binary to PATH

You're reading this guide because `steel init --agent` just ran — the native
binary is already installed at `~/.steel/bin/steel`. The installer appended
`~/.steel/bin` to the user's shell rc file, but **that change won't affect
your current session**. Add it to PATH now so the rest of this guide works
as written:

```bash
export PATH="$HOME/.steel/bin:$PATH"
steel --version
```

If for some reason the binary is not present, install it:

```bash
curl -fsS https://setup.steel.dev | sh
export PATH="$HOME/.steel/bin:$PATH"
```

### Step 2 — Authenticate

The `--agent` flag you just ran only prints this guide. Now run the real
thing:

```bash
steel init
```

This command is idempotent: it will no-op if the user is already logged in.

On first run it will:

1. Print an authentication URL and open a browser to it.
2. Wait (up to 5 minutes) for the user to sign in.
3. Persist the resulting API key to the user's Steel config.

**Your job during this step:** show the user the URL Steel printed and say
something like *"Please click this link to sign in. I'll wait here."* Do not
continue until Steel prints `Authentication successful!`.

If the user is in a sandboxed environment without a local browser (e.g.
Claude Code Web, Replit, GitHub Codespaces without port-forwarding), the
authentication URL will still work — the user just needs to open it from any
device they have a browser on. Tell them this explicitly if the browser does
not open automatically.

### Step 3 — Preflight check

Verify auth + API connectivity:

```bash
steel doctor --preflight
```

All checks should report `pass`. If any report `fail`, surface the `fix`
column of the output to the user verbatim and stop.

### Step 4 — Smoke test

Confirm Steel can actually run a browser for you:

```bash
steel browser start --session hello
steel browser navigate https://example.com --session hello
steel browser get title --session hello
steel browser stop --session hello
```

If the last command returns `"Example Domain"` as the page title, Steel is
ready. Tell the user:

> Steel is set up and ready. You can now ask me to do any web task and I'll
> use Steel to handle pages that need JavaScript, login flows, or CAPTCHA
> bypass.

---

## Tooling overview (cheatsheet for after setup)

| Task | Command |
|---|---|
| Scrape a page to Markdown | `steel scrape <url>` |
| Take a screenshot | `steel screenshot <url>` |
| Save a page as PDF | `steel pdf <url>` |
| Start an interactive browser | `steel browser start --session <name>` |
| Navigate / click / fill in a session | `steel browser navigate / click / fill` |
| See all sessions | `steel browser sessions` |
| End a session | `steel browser stop --session <name>` |
| Check environment health | `steel doctor` |

Full reference: https://docs.steel.dev

---

## Common failure modes

| Symptom | What to do |
|---|---|
| `steel: command not found` | Repeat Step 1: `export PATH="$HOME/.steel/bin:$PATH"`. If `~/.steel/bin/steel` doesn't exist, re-run `curl -fsS https://setup.steel.dev \| sh`. |
| Browser does not open on `steel init` | Show the URL to the user and ask them to open it manually on any device. Wait for `Authentication successful!`. |
| `steel doctor --preflight` reports `auth: fail` | Run `steel login` again. If it still fails, the user's API key may have been revoked — direct them to https://app.steel.dev/settings/api-keys. |
| `steel doctor --preflight` reports `api: fail` | Usually a network issue. Ask the user to check their internet connection. Do **not** retry in a tight loop. |
| `steel browser start` hangs | Run `steel browser sessions` to see existing sessions. Stop stale ones with `steel browser stop --all` and retry. |

---

## What this command is NOT

- It is **not** a project scaffolder. For templates, the user can run
  `steel forge` themselves, but don't suggest it unsolicited.
- It does **not** create a Steel account for the user. Steel accounts are
  created through the normal web sign-up flow at https://app.steel.dev. If
  the user does not have an account, the login page will prompt them to
  create one — that's fine, the flow is the same.
- It does **not** install any MCP server (Steel doesn't ship one). You
  interact with Steel by calling `steel` subcommands from your shell tool.

When this guide ends, you are expected to actually run `steel init` (and the
subsequent verification commands) in the user's shell. Reading this guide is
step zero, not the whole process.
