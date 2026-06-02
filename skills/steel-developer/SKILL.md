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
- Node examples import the SDK package as `import Steel from "steel-sdk"`; the SDK source repo is `steel-dev/steel-node`
- Node SDK constructor uses `steelAPIKey`
- Python SDK constructor uses `steel_api_key`
- Always release sessions in cleanup
- Construct the WebSocket URL explicitly as `wss://connect.steel.dev?apiKey=...&sessionId=...`
- Reuse the default browser context after connecting; do not create a new context unless the user explicitly needs isolation
- Do not leak raw credentials into prompts, page scripts, logs, or generated examples when Steel Credentials can be used instead
- For exact API fields, response shapes, and SDK method signatures, consult the API reference or SDK types; do not guess from memory
- When retrieving Steel docs via shell, use `curl -sSfL` so redirects are followed and HTTP failures are visible
- Never use stale SDK patterns: `steelClient`, `client.createSession`, `client.releaseSession`, `Steel(api_key=...)`, `session.websocketUrl`, `session.websocket_url`, Playwright `chromium.connect({ wsEndpoint })`, or top-level `browser.newPage()` for the default Steel session
- For API helpers beyond basic sessions, include a short source-of-truth note: "Check the Steel API reference or SDK types for exact method signatures and response fields."

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

TypeScript Playwright shape:

```ts
const client = new Steel({ steelAPIKey });
const session = await client.sessions.create();
const cdpUrl = `wss://connect.steel.dev?apiKey=${steelAPIKey}&sessionId=${session.id}`;
const browser = await chromium.connectOverCDP(cdpUrl);
const context = browser.contexts()[0];
const page = context.pages()[0] ?? (await context.newPage());
try {
  await page.goto("https://example.com");
} finally {
  await browser.close();
  await client.sessions.release(session.id);
}
```

Python Playwright shape:

```python
steel_api_key = os.environ["STEEL_API_KEY"]
client = Steel(steel_api_key=steel_api_key)
session = client.sessions.create()
cdp_url = f"wss://connect.steel.dev?apiKey={steel_api_key}&sessionId={session.id}"
browser = await playwright.chromium.connect_over_cdp(cdp_url)
context = browser.contexts[0]
page = context.pages[0] if context.pages else await context.new_page()
try:
    await page.goto("https://example.com")
finally:
    await browser.close()
    await playwright.stop()
    client.sessions.release(session.id)
```

### 3. Use profiles correctly

- First session: set `persistProfile: true` to create and save a profile
- Reuse later: pass `profileId`
- Keep evolving stored state: pass both `profileId` and `persistProfile: true`
- Use profiles for cookies, auth state, extensions, settings, and browser context reuse across sessions
- `sessionContext` is lighter cookie/localStorage transfer; capture it with `client.sessions.context(session.id)` before releasing the source session, then pass `sessionContext` into a new session

Profile shape:

```ts
const firstSession = await client.sessions.create({ persistProfile: true });
// Log in or configure browser state, then release so Steel persists it.
await client.sessions.release(firstSession.id);

const secondSession = await client.sessions.create({ profileId: firstSession.profileId });
const updatedSession = await client.sessions.create({
  profileId: firstSession.profileId,
  persistProfile: true,
});
```

Auth context shape:

```ts
const sessionContext = await client.sessions.context(sourceSession.id);
await client.sessions.release(sourceSession.id);
const nextSession = await client.sessions.create({ sessionContext });
```

### 4. Use credentials correctly

- Store credentials once per origin and namespace
- Start the session with matching `namespace` and `credentials: {}`
- Navigate to the login page and allow time for injection
- Verify login success with a page assertion instead of assuming it worked
- Credential values must come from environment variables such as `APP_PASSWORD` and optional `APP_TOTP_SECRET`; never write placeholders like `your_password_here` into examples
- Do not fill username/password/TOTP with Playwright when using Steel Credentials; create credentials, enable them on the session, navigate, wait, and assert success
- Do not invent a TOTP retrieval helper; store `totpSecret` in `client.credentials.create` and let Steel inject it

Credential shape:

```ts
await client.credentials.create({
  origin: "https://app.example.com",
  namespace: "example:fred",
  value: {
    username: process.env.APP_USERNAME!,
    password: process.env.APP_PASSWORD!,
    totpSecret: process.env.APP_TOTP_SECRET,
  },
});

const session = await client.sessions.create({
  namespace: "example:fred",
  credentials: {},
});
```

## API shortcuts

- Browser Tools are the correct choice for one-shot scrape/screenshot/PDF with no reusable browser state; do not create a session for these
- Node one-shot scrape shape: `await client.scrape({ url, format: ["markdown"], screenshot: true, pdf: true })`, then read `result.metadata.title`; always say "Check the Steel API reference or SDK types for exact response fields."
- Files: upload or mount files into the Steel session first, then use the Steel-returned file path for Playwright/Puppeteer file inputs; always say "Check the Steel API reference or SDK types for exact Files API method signatures."
- Extensions: Chrome extensions are not Files API uploads; use Extensions API (`client.extensions.upload` or list existing extensions) to get a specific `extensionId`, then start a session with `extensionIds: [extensionId]`; explicitly say extensions are organization-scoped, initialize at session start, and exact method signatures come from the Steel API reference or SDK types; do not use `all_ext` unless explicitly requested
- CAPTCHAs: show the `curl -sSfL https://api.steel.dev/v1/details` plan check first; after plan gating, create the session with `solveCaptcha: true` and `stealthConfig: { autoCaptchaSolving: false }` for manual mode; poll `client.sessions.captchas.status(session.id)`, solve targeted tasks with `client.sessions.captchas.solve(session.id, { taskId })`, and handle failed statuses/timeouts
- Hobby plus BYOP: use `useProxy: { server: process.env.PROXY_SERVER! }`, not Steel-managed `useProxy: true`; BYOP is separate from Steel-managed proxy bandwidth; always say to check plan before enabling Steel-managed proxies and to retry/fallback on transient proxy errors such as `ERR_TUNNEL_CONNECTION_FAILED`

Manual CAPTCHA shape:

```ts
const session = await client.sessions.create({
  solveCaptcha: true,
  stealthConfig: { autoCaptchaSolving: false },
});
const states = await client.sessions.captchas.status(session.id);
await client.sessions.captchas.solve(session.id, { taskId });
```

## Source-of-truth links

- API reference: https://steel.apidocumentation.com/api-reference
- Sessions create: https://steel.apidocumentation.com/api-reference#tag/sessions/post/v1/sessions
- Node SDK/types: https://github.com/steel-dev/steel-node
- Python SDK/types: https://github.com/steel-dev/steel-python
- Docs index: `curl -sSfL https://docs.steel.dev/llms.txt`
- Full docs bundle: `curl -sSfL https://docs.steel.dev/llms-full.txt`
- Single docs page: `curl -sSfL https://docs.steel.dev/llms.mdx/<page-path>`
- In answers about exact API or SDK lookup, include those `llms.txt`, `llms-full.txt`, or `/llms.mdx/<page-path>` docs-fetching options by name, not just the API reference

## Ecosystem routing

- Direct Playwright/Puppeteer is the deterministic baseline
- Stagehand is the TypeScript path for natural-language browser actions; every Stagehand recommendation should also mention that direct Playwright/Puppeteer remains the deterministic baseline; docs: https://docs.steel.dev/integrations/stagehand and https://docs.steel.dev/cookbook/stagehand
- Browser Use is the Python path for an LLM browser-agent loop; pass the explicit Steel CDP URL into `BrowserSession`, expose CAPTCHA status helpers when needed, and release the Steel session in cleanup
- For typed agent products, route to Vercel AI SDK, OpenAI Agents SDK, Mastra, AgentKit, LangGraph, Pydantic AI, Agno, CrewAI, Notte, or Magnitude based on language and product needs; keep Steel session lifecycle explicit

## Output guidance

- Default to TypeScript examples for Puppeteer
- Default to TypeScript or Python examples for Playwright based on the user's stack
- Mention Browser Use for Python agent loops and Stagehand for TypeScript natural-language browser actions, but keep direct Playwright/Puppeteer examples as the baseline
- When the user asks "what can Steel do", explain scrape, browser sessions, proxies, CAPTCHA solving, profiles, credentials, extensions, files, and agent integrations
- When asked to implement, emit minimal runnable code with cleanup
- When the user asks for docs/source-of-truth lookup, mention `llms.txt`, `llms-full.txt`, and `/llms.mdx/<page-path>` with `curl -sSfL`
- When recommending Stagehand, also mention direct Playwright/Puppeteer as the deterministic baseline and include the Stagehand integration/cookbook links
- When explaining profiles, always show all three profile calls: create with `persistProfile`, reuse with `profileId`, and update with both `profileId` plus `persistProfile`; also state `sessionContext` must be captured before releasing the source session
- When answering Browser Tools, Files, or Extensions questions, always include one sentence pointing to the Steel API reference or SDK types for exact fields/method signatures
- When answering BYOP/proxy questions, always mention plan-checking before Steel-managed proxies and retry/fallback for `ERR_TUNNEL_CONNECTION_FAILED`
- When the user asks the agent to browse a site now, do not offer a reusable script; route to `steel-browser` and suggest `steel scrape <url>` or `steel browser ...`
