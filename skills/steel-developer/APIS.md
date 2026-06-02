# Steel APIs

Use this file to choose the right Steel API and avoid common implementation mistakes. Do not copy complete request or response schemas into generated answers unless the user asks for them.

## Source Of Truth

- API reference: https://steel.apidocumentation.com/api-reference
- Sessions create endpoint: https://steel.apidocumentation.com/api-reference#tag/sessions/post/v1/sessions
- Node SDK: https://github.com/steel-dev/steel-node
- Python SDK: https://github.com/steel-dev/steel-python

Use this skill for routing, patterns, and gotchas. Use the API reference and SDK types for exact option names, request shapes, response fields, and method signatures.

## Fetching Steel Docs

When fetching Steel docs with `curl`, follow redirects.

Use:

```bash
curl -sSfL https://docs.steel.dev/llms.txt
curl -sSfL https://docs.steel.dev/llms-full.txt
curl -sSfL https://docs.steel.dev/llms.mdx/overview/sessions-api/quickstart
```

Rules:

- Always include `-L`; docs URLs may redirect.
- Prefer `-sSfL` for agent use: silent progress, fail on HTTP errors, follow redirects.
- Use `llms.txt` for the docs index and quick reference.
- Use `llms-full.txt` for broad offline/context loading.
- Use `/llms.mdx/<page-path>` for a single exact docs page.
- Use the API reference for exact API shapes.
- Use SDK repos and local installed types for exact SDK method names and option types.

## API Routing

| Need | Use | Reference |
| --- | --- | --- |
| Create and release a browser session | Sessions API | https://steel.apidocumentation.com/api-reference#tag/sessions |
| Connect Playwright or Puppeteer | Sessions API plus CDP URL | https://docs.steel.dev/integrations/playwright |
| One-shot scrape, screenshot, or PDF | Browser Tools | https://docs.steel.dev/overview/browser-tools/overview |
| Persist full browser state | Profiles API | https://steel.apidocumentation.com/api-reference#tag/profiles |
| Reuse cookies/localStorage once | Session context | https://docs.steel.dev/overview/sessions-api/reusing-auth-context |
| Secure login injection | Credentials API | https://steel.apidocumentation.com/api-reference#tag/credentials |
| CAPTCHA status and solving | CAPTCHAs API | https://steel.apidocumentation.com/api-reference#tag/captchas |
| Steel-managed or BYOP proxying | Session `useProxy` | https://docs.steel.dev/overview/stealth/proxies |
| Upload/download files | Files API | https://steel.apidocumentation.com/api-reference#tag/files |
| Add Chrome extensions | Extensions API | https://steel.apidocumentation.com/api-reference#tag/extensions |
| Embed live or past session viewers | Session embeds | https://docs.steel.dev/overview/sessions-api/embed-sessions |
| Review agent activity timeline | Agent traces | https://docs.steel.dev/overview/agent-traces/overview |
| Mobile viewport/fingerprints | Mobile mode | https://docs.steel.dev/overview/sessions-api/mobile-mode |
| Region placement | Multi-region | https://docs.steel.dev/overview/sessions-api/multi-region |
| User handoff/control | Human-in-the-loop | https://docs.steel.dev/overview/sessions-api/human-in-the-loop |

## Sessions

Use sessions for multi-step browser automation, auth state, file uploads/downloads, extensions, proxies, CAPTCHAs, and any workflow that needs Playwright or Puppeteer control.

Gotchas:

- Release sessions in cleanup instead of waiting for timeout.
- Construct the CDP URL explicitly as `wss://connect.steel.dev?apiKey=...&sessionId=...`.
- Reuse the default browser context after connecting.
- Set `timeout` at creation time; do not assume live session timeouts can be edited.
- Use SDK types or the API reference before adding less-common create options.

References:

- Create session: https://steel.apidocumentation.com/api-reference#tag/sessions/post/v1/sessions
- Session lifecycle: https://docs.steel.dev/overview/sessions-api/session-lifecycle
- Quickstart: https://docs.steel.dev/overview/sessions-api/quickstart

## Browser Tools

Use Browser Tools for one-shot reads or artifacts when you do not need long-running session state.

Use cases:

- `client.scrape` for HTML, cleaned HTML, Markdown, Readability, metadata, and links.
- `client.screenshot` for a hosted PNG.
- `client.pdf` for a hosted PDF.
- `client.scrape` with `screenshot: true` or `pdf: true` when content and artifact should be captured together.

Gotchas:

- Browser Tools are stateless and rate limited; use sessions for cookies, multi-step navigation, file upload, extensions, or proxy geotargeting.
- `useProxy: true` on Browser Tools requires a plan that supports Steel-managed proxies.
- Use `delay` for hydrated client-rendered pages.

Reference: https://docs.steel.dev/overview/browser-tools/overview

## Profiles And Auth Context

Prefer profiles for reusable browser identity. Use auth context for a lighter transfer of cookies/localStorage between live sessions.

Profiles:

- Use `persistProfile` on a first session to create a profile.
- Reuse with `profileId`.
- Pass both `profileId` and `persistProfile` when the session should update stored state.
- Profiles can include cookies, auth state, extensions, credentials, and browser settings.
- Profiles have size and retention limits; check docs for current details.

Auth context:

- Capture context before releasing the source session.
- Treat captured context as sensitive data.
- Pass captured context into a new session as `sessionContext`.
- Prefer profiles when you need full browser user data or repeated reuse.

References:

- Profiles API: https://docs.steel.dev/overview/profiles-api/overview
- Reusing context and auth: https://docs.steel.dev/overview/sessions-api/reusing-auth-context

## Credentials

Use Credentials API when login secrets should not be exposed to the agent, generated code logs, page scripts, or prompts.

Gotchas:

- Create credentials once per `origin` and `namespace`.
- Session `namespace` must exactly match the credential namespace.
- Pass `credentials: {}` to enable default injection options.
- Default options include `autoSubmit`, `blurFields`, and `exactOrigin`; consult SDK types/API docs for exact names.
- Navigate to the login page, wait for injection, then assert successful login.
- Use `totpSecret` for TOTP when supported by the target flow.

Reference: https://docs.steel.dev/overview/credentials-api/overview

## CAPTCHAs

Enable CAPTCHA solving only when the plan supports it.

Patterns:

- Auto solve: create session with `solveCaptcha: true` / `solve_captcha=True`.
- Detect but manually solve: enable solving but set `stealthConfig.autoCaptchaSolving` false.
- Poll status with `client.sessions.captchas.status(session.id)`.
- Manually solve all detected CAPTCHAs or target a `taskId`, `url`, or `pageId`.
- Use image CAPTCHA solving only when you can provide stable XPath selectors.

Gotchas:

- Handle failed statuses and timeouts explicitly.
- CAPTCHA solving may need proxies and natural pacing.
- Do not silently enable paid solving for Hobby plans.

References:

- CAPTCHAs API: https://docs.steel.dev/overview/captchas-api/overview
- CAPTCHA solving overview: https://docs.steel.dev/overview/stealth/captcha-solving

## Proxies

Start without proxies, then add them when a target blocks Steel's default datacenter IPs or requires geolocation.

Patterns:

- Default: no proxy, available on all plans.
- Steel-managed proxy: `useProxy: true` / `use_proxy=True`, plan-gated.
- Geotargeting: prefer country-level before state/city-level targeting.
- BYOP: pass a proxy `server`; this is separate from Steel-managed proxy bandwidth.

Gotchas:

- Retry transient proxy errors like `ERR_TUNNEL_CONNECTION_FAILED`.
- If using BYOP, keep proxy credentials in environment variables.
- Some domains may be restricted by proxy provider compliance policies.

Reference: https://docs.steel.dev/overview/stealth/proxies

## Files

Use Files API when a session needs uploaded inputs or must retrieve downloaded outputs.

Patterns:

- Upload a file into an active session before setting a file input.
- Upload once to global files, then mount into sessions by path.
- Download one file or the session archive after automation.
- Use CDP or framework-specific file upload APIs with the file path returned by Steel.

Gotchas:

- File path formats can differ between SDK helpers and raw HTTP paths; consult the API reference.
- Session files are tied to the session environment and can be promoted/backed up after release.
- Browser Use download flows may need explicit download path handling.

Reference: https://docs.steel.dev/overview/files-api/overview

## Extensions

Use Extensions API when a Chrome extension should be installed into a Steel session.

Patterns:

- Upload an extension from `.zip` / `.crx` or a Chrome Web Store URL.
- List extensions to retrieve IDs.
- Start sessions with `extensionIds` / `extension_ids`.
- Use `all_ext` only when all installed extensions should load.

Gotchas:

- Upload/update/delete extensions are organization-scoped operations.
- Extensions initialize at session start; create a new session after changing extension configuration.
- Validate extension permissions and manifest before upload.

Reference: https://docs.steel.dev/overview/extensions-api/overview

## Session UX APIs

Use these when building user-facing products on top of Steel sessions.

Patterns:

- Use live session viewer URLs for debugging and human takeover.
- Use past session embeds for replay or post-run inspection.
- Use agent traces for timeline/debug views and exports.
- Use human-in-the-loop controls when a user should take over sensitive steps.
- Use mobile mode, dimensions, and multi-region when the target requires a specific device or location profile.

References:

- Embed sessions: https://docs.steel.dev/overview/sessions-api/embed-sessions
- Live sessions: https://docs.steel.dev/overview/sessions-api/embed-sessions/live-sessions
- Past sessions: https://docs.steel.dev/overview/sessions-api/embed-sessions/past-sessions
- Agent traces: https://docs.steel.dev/overview/agent-traces/overview
- Human-in-the-loop: https://docs.steel.dev/overview/sessions-api/human-in-the-loop
- Mobile mode: https://docs.steel.dev/overview/sessions-api/mobile-mode
- Multi-region: https://docs.steel.dev/overview/sessions-api/multi-region
