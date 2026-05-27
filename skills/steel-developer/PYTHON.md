# Steel Python

Use this file for Python, Playwright Python, Browser Use, and Python browser-agent tasks.

## Install and configure

```bash
pip install steel-sdk playwright
```

Use `STEEL_API_KEY` from the environment. Do not hardcode API keys or target-site credentials.

## Check the plan before paid stealth features

Run this before enabling `solve_captcha` or Steel-managed `use_proxy`:

```bash
curl -sf "https://api.steel.dev/v1/details" \
  -H "steel-api-key: $STEEL_API_KEY" | jq -r '.plan'
```

If the plan is `hobby`, do not enable `solve_captcha` or Steel-managed `use_proxy`. Use default datacenter networking, BYOP, or ask the user to upgrade.

## Create a session

```python
import os
from steel import Steel

steel_api_key = os.environ["STEEL_API_KEY"]
client = Steel(steel_api_key=steel_api_key)

session = client.sessions.create(
    # Only enable these after checking the plan.
    # solve_captcha=True,
    # use_proxy=True,
)

cdp_url = f"wss://connect.steel.dev?apiKey={steel_api_key}&sessionId={session.id}"
print(f"Live view: {session.session_viewer_url}")
```

## Connect with Playwright

Reuse Steel's default context and the page already opened by the session. Do not call `browser.new_context()` unless the user explicitly needs a separate context.

```python
from playwright.async_api import async_playwright

playwright = await async_playwright().start()
browser = await playwright.chromium.connect_over_cdp(cdp_url)
context = browser.contexts[0]
page = context.pages[0] if context.pages else await context.new_page()

try:
    await page.goto("https://example.com", wait_until="domcontentloaded")
finally:
    await browser.close()
    await playwright.stop()
    client.sessions.release(session.id)
```

Puppeteer is not the Python path. Use Playwright for Python browser control or Browser Use for an agent loop.

## Build automations

Prefer stable app semantics over brittle CSS selectors.

```python
await page.goto("https://app.example.com/login", wait_until="domcontentloaded")
await page.get_by_label("Email").fill("user@example.com")
await page.get_by_label("Password").fill(os.environ["APP_PASSWORD"])
await page.get_by_role("button", name="Sign in").click()
await page.wait_for_url("**/dashboard")

rows = await page.locator("[data-row]").evaluate_all(
    "elements => elements.map(element => element.textContent.trim()).filter(Boolean)"
)
```

For synchronous scripts, use `playwright.sync_api`, but keep the same Steel pattern: create session, connect over CDP, reuse `browser.contexts[0]`, release the session in cleanup.

## Use profiles for reusable state

Profiles persist browser user data such as cookies, auth state, extensions, credentials, and settings.

```python
first_session = client.sessions.create(persist_profile=True)
# Run the login/setup flow, then release so Steel can persist the profile.
client.sessions.release(first_session.id)

second_session = client.sessions.create(
    profile_id=first_session.profile_id,
    persist_profile=True,
)
```

Use `profile_id` to load state. Add `persist_profile=True` when the session should update the stored profile after it releases.

## Use credentials safely

Create credentials once per `origin` and `namespace`, then create sessions with the same namespace and `credentials` enabled.

```python
client.credentials.create(
    origin="https://app.example.com",
    namespace="example:fred",
    value={
        "username": "fred@example.com",
        "password": os.environ["APP_PASSWORD"],
        # "totpSecret": os.environ["APP_TOTP_SECRET"],
    },
)

session = client.sessions.create(
    namespace="example:fred",
    credentials={
        "autoSubmit": True,
        "blurFields": True,
        "exactOrigin": True,
    },
)
```

After connecting, navigate to the login page, wait for injection, then assert login success.

```python
await page.goto("https://app.example.com/login", wait_until="domcontentloaded")
await page.wait_for_timeout(2_000)
await page.wait_for_url("**/dashboard", timeout=30_000)
```

## Track CAPTCHA solves

Enable solving only when the plan supports it.

```python
session = client.sessions.create(
    solve_captcha=True,
)
```

Use `sessions.captchas.status` to monitor progress.

```python
import asyncio
import time

ACTIVE_STATUSES = {"detected", "validating", "solving"}
FAILED_STATUSES = {"failed_to_detect", "failed_to_solve", "validation_failed"}


def field(value, name, default=None):
    if isinstance(value, dict):
        return value.get(name, default)
    return getattr(value, name, default)


def summarize_captcha_states(states):
    tasks = [task for state in states for task in (field(state, "tasks", []) or [])]
    active_tasks = [task for task in tasks if field(task, "status") in ACTIVE_STATUSES]
    failed_tasks = [task for task in tasks if field(task, "status") in FAILED_STATUSES]
    solved_tasks = [task for task in tasks if field(task, "status") == "solved"]
    active_pages = [state for state in states if field(state, "isSolvingCaptcha", False)]

    return {
        "pages": len(states),
        "active_pages": len(active_pages),
        "active_tasks": len(active_tasks),
        "solved_tasks": len(solved_tasks),
        "failed_tasks": len(failed_tasks),
    }


async def wait_for_captcha_solution(client, session_id, timeout_seconds=90):
    deadline = time.monotonic() + timeout_seconds

    while time.monotonic() < deadline:
        states = client.sessions.captchas.status(session_id)
        summary = summarize_captcha_states(states)
        print(summary)

        if summary["failed_tasks"]:
            raise RuntimeError("CAPTCHA solve failed")

        if summary["active_pages"] == 0 and summary["active_tasks"] == 0:
            return summary

        await asyncio.sleep(1)

    raise TimeoutError("Timed out waiting for CAPTCHA solving")
```

Call it after navigating to pages that may trigger CAPTCHA challenges:

```python
await page.goto("https://example.com/protected", wait_until="domcontentloaded")
await wait_for_captcha_solution(client, session.id)
```

## Stealth and proxies

Start simple. Add stealth features only when needed and plan-compatible.

```python
session = client.sessions.create(
    use_proxy=True,
    solve_captcha=True,
)
```

Use broad geotargeting before narrow city-level targeting.

```python
session = client.sessions.create(
    use_proxy={
        "geolocation": {"country": "US"},
    },
)
```

Use BYOP when the user provides their own proxy server or the plan does not include Steel-managed proxies.

```python
session = client.sessions.create(
    use_proxy={
        "server": os.environ["PROXY_SERVER"],
    },
)
```

Recommendations:

- Establish a baseline without proxies first
- Reuse profiles for sites where cookies and reputation help
- Add natural delays instead of rapid repeated actions
- Retry transient proxy errors such as `ERR_TUNNEL_CONNECTION_FAILED`
- Prefer country-level targeting unless location precision is required

## Bigger Python frameworks

Use direct Playwright for deterministic tools. Use Browser Use when the user wants an LLM agent loop that navigates, fills forms, and extracts data.

```python
from browser_use import Agent, BrowserSession
from browser_use.llm import ChatOpenAI

agent = Agent(
    task="Find the latest news on Steel.dev",
    llm=ChatOpenAI(model="gpt-5", api_key=os.environ["OPENAI_API_KEY"]),
    browser_session=BrowserSession(cdp_url=cdp_url),
)

result = await agent.run()
```

For CAPTCHA-heavy Browser Use flows, expose `wait_for_captcha_solution` as a Browser Use tool and instruct the agent to call it when it sees a CAPTCHA.

- Browser Use integration: https://docs.steel.dev/integrations/browser-use
- Browser Use recipe: https://docs.steel.dev/cookbook/browser-use
- Browser Use CAPTCHA recipe: https://docs.steel.dev/cookbook/browser-use-captcha-auto
- Playwright integration: https://docs.steel.dev/integrations/playwright
