---
name: steel-developer
description: Builds software on Steel cloud browsers with SDK/REST sessions, Playwright, Puppeteer, Stagehand, Browser Use, credentials, profiles, proxies, and CAPTCHA APIs. Use ONLY when writing, debugging, or explaining application code, scripts, reusable workflows, examples, or docs that run on Steel. Do not use for live web browsing, extraction, screenshots, or form filling performed by the agent; use the steel-browser CLI skill for that.
license: MIT
compatibility: opencode
metadata:
  owner: steel
  workflow: education
---

# Steel Developer

## Skill boundary

- Use this skill when the user wants code or implementation guidance they will run later: SDK setup, Steel sessions, Playwright/Puppeteer connections, Stagehand, Browser Use, credentials, profiles, proxies, CAPTCHA APIs, or reusable automations
- Do not use this skill when the user wants the agent to browse a site now, extract content now, capture a screenshot/PDF now, or interact with a form now; use the `steel-browser` CLI skill for those live tasks
- If a task needs both, use `steel-browser` only for live reconnaissance and use this skill for the final reusable code
- Do not copy SDK examples from the `steel-browser` CLI skill; this skill is the source of truth for building on Steel

## Route by language

Load the relevant reference before writing code:

- TypeScript, JavaScript, Node.js, Playwright, Puppeteer, or Stagehand: read [TYPESCRIPT.md](TYPESCRIPT.md)
- Python, Playwright Python, Browser Use, or Python agents: read [PYTHON.md](PYTHON.md)
- First-party Steel APIs, exact docs/API/SDK lookup, browser tools, files, extensions, auth context, embeds, traces, mobile mode, or multi-region: read [APIS.md](APIS.md)
- Framework choice, Stagehand, Browser Use, computer-use integrations, typed agent frameworks, or coding-agent integrations: read [ECOSYSTEM.md](ECOSYSTEM.md)
- Mixed-language or ambiguous stack: inspect the repo, then read the matching file or ask one short question

## Non-negotiables

- Auth env var is `STEEL_API_KEY`
- Node SDK constructor uses `steelAPIKey`
- Python SDK constructor uses `steel_api_key`
- Always release sessions in cleanup
- Construct the WebSocket URL explicitly as `wss://connect.steel.dev?apiKey=...&sessionId=...`
- Reuse the default browser context after connecting; do not create a new context unless the user explicitly needs isolation
- Do not leak raw credentials into prompts, page scripts, logs, or generated examples when Steel Credentials can be used instead
- For exact API fields, response shapes, and SDK method signatures, consult the API reference or SDK types; do not guess from memory
- When retrieving Steel docs via shell, use `curl -sSfL` so redirects are followed and HTTP failures are visible

## Plan gate

Before enabling `solveCaptcha` or Steel-managed `useProxy`, determine the plan:

```bash
curl -sSfL "https://api.steel.dev/v1/details" \
  -H "steel-api-key: $STEEL_API_KEY" | jq -r '.plan'
```

- Hobby plans do not have CAPTCHA solves or Steel-managed proxies
- If the plan is Hobby, do not silently enable `solveCaptcha` or `useProxy: true`; use default datacenter networking, BYOP if appropriate, or ask the user to upgrade
- BYOP is separate from Steel-managed proxy bandwidth and can be used when the user provides their own proxy server

## Workflows

### 1. Choose the right Steel primitive

- One-shot live web task for the agent: use `steel-browser`, not this skill
- Scrape-only feature inside an app or script: use REST scrape or SDK scrape
- Browser automation task: create a session, connect Playwright or Puppeteer, do the work, release the session
- Reusable authenticated state: use profiles
- Secure login without exposing secrets to the agent: use credentials
- File upload/download workflows: use Steel Files APIs and a session-backed browser
- Browser extension workflows: upload/manage extensions, then start sessions with extension IDs
- Anti-bot mitigation: start with Steel defaults, then add profiles, natural pacing, managed proxies or BYOP, and CAPTCHA solving when the plan allows it

### 2. Build a session-based browser tool

- Create a Steel client
- Create a session with only the options needed, such as `timeout`, `useProxy`, `solveCaptcha`, `profileId`, `persistProfile`, `namespace`, or `credentials`
- Connect the browser library to `wss://connect.steel.dev?apiKey=<key>&sessionId=<id>`
- Reuse the default context and existing page
- Close the browser and release the Steel session in cleanup

### 3. Use profiles correctly

- First session: set `persistProfile: true` to create and save a profile
- Reuse later: pass `profileId`
- Keep evolving stored state: pass both `profileId` and `persistProfile: true`
- Use profiles for cookies, auth state, extensions, settings, and browser context reuse across sessions

### 4. Use credentials correctly

- Store credentials once per origin and namespace
- Start the session with matching `namespace` and `credentials: {}`
- Navigate to the login page and allow time for injection
- Verify login success with a page assertion instead of assuming it worked

## Output guidance

- Default to TypeScript examples for Puppeteer
- Default to TypeScript or Python examples for Playwright based on the user's stack
- Mention Browser Use for Python agent loops and Stagehand for TypeScript natural-language browser actions, but keep direct Playwright/Puppeteer examples as the baseline
- When the user asks "what can Steel do", explain scrape, browser sessions, proxies, CAPTCHA solving, profiles, credentials, extensions, files, and agent integrations
- When asked to implement, emit minimal runnable code with cleanup
