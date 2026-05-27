# Steel TypeScript and JavaScript

Use this file for Node.js, TypeScript, JavaScript, Playwright, Puppeteer, and Stagehand tasks.

## Install and configure

```bash
npm install steel-sdk playwright puppeteer-core
```

Use `STEEL_API_KEY` from the environment. Do not hardcode API keys or target-site credentials.

## Check the plan before paid stealth features

Run this before enabling `solveCaptcha` or Steel-managed `useProxy`:

```bash
curl -sf "https://api.steel.dev/v1/details" \
  -H "steel-api-key: $STEEL_API_KEY" | jq -r '.plan'
```

If the plan is `hobby`, do not enable `solveCaptcha` or Steel-managed `useProxy`. Use default datacenter networking, BYOP, or ask the user to upgrade.

## Create a session

```ts
import Steel from "steel-sdk";

const steelAPIKey = process.env.STEEL_API_KEY;
if (!steelAPIKey) throw new Error("STEEL_API_KEY is required");

const client = new Steel({ steelAPIKey });

const session = await client.sessions.create({
  timeout: 10 * 60 * 1000,
  // Only enable these after checking the plan.
  // solveCaptcha: true,
  // useProxy: true,
});

const cdpUrl = `wss://connect.steel.dev?apiKey=${steelAPIKey}&sessionId=${session.id}`;
console.log(`Live view: ${session.sessionViewerUrl}`);
```

## Connect with Playwright

Reuse Steel's default context and the page already opened by the session. Do not call `browser.newContext()` unless the user explicitly needs a separate context.

```ts
import { chromium } from "playwright";

const browser = await chromium.connectOverCDP(cdpUrl);
const context = browser.contexts()[0];
const page = context.pages()[0] ?? (await context.newPage());

try {
  await page.goto("https://example.com", { waitUntil: "domcontentloaded" });
} finally {
  await browser.close();
  await client.sessions.release(session.id);
}
```

## Connect with Puppeteer

Use the default browser context. Avoid `createBrowserContext()` for normal Steel sessions.

```ts
import puppeteer from "puppeteer-core";

const browser = await puppeteer.connect({ browserWSEndpoint: cdpUrl });
const context = browser.defaultBrowserContext();
const page = (await context.pages())[0] ?? (await context.newPage());

try {
  await page.goto("https://example.com", { waitUntil: "domcontentloaded" });
} finally {
  await browser.close();
  await client.sessions.release(session.id);
}
```

## Build automations

Prefer stable app semantics over brittle CSS selectors.

### Playwright automation

```ts
await page.goto("https://app.example.com/login", { waitUntil: "domcontentloaded" });
await page.getByLabel(/email/i).fill("user@example.com");
await page.getByLabel(/password/i).fill(process.env.APP_PASSWORD!);
await page.getByRole("button", { name: /sign in/i }).click();
await page.waitForURL(/dashboard|home/);

const rows = await page.locator("[data-row]").evaluateAll((elements) =>
  elements.map((element) => element.textContent?.trim()).filter(Boolean),
);
```

### Puppeteer automation

```ts
await page.goto("https://app.example.com/login", { waitUntil: "networkidle2" });
await page.waitForSelector("input[name=email]");
await page.type("input[name=email]", "user@example.com");
await page.type("input[name=password]", process.env.APP_PASSWORD!);
await Promise.all([
  page.waitForNavigation({ waitUntil: "networkidle2" }),
  page.click("button[type=submit]"),
]);

const title = await page.$eval("h1", (element) => element.textContent?.trim());
```

## Use profiles for reusable state

Profiles persist browser user data such as cookies, auth state, extensions, credentials, and settings.

```ts
const firstSession = await client.sessions.create({ persistProfile: true });
// Run the login/setup flow, then release so Steel can persist the profile.
await client.sessions.release(firstSession.id);

const secondSession = await client.sessions.create({
  profileId: firstSession.profileId,
  persistProfile: true,
});
```

Use `profileId` to load state. Add `persistProfile: true` when the session should update the stored profile after it releases.

## Use credentials safely

Create credentials once per `origin` and `namespace`, then create sessions with the same namespace and `credentials` enabled.

```ts
await client.credentials.create({
  origin: "https://app.example.com",
  namespace: "example:fred",
  value: {
    username: "fred@example.com",
    password: process.env.APP_PASSWORD!,
    // totpSecret: process.env.APP_TOTP_SECRET,
  },
});

const session = await client.sessions.create({
  namespace: "example:fred",
  credentials: {
    autoSubmit: true,
    blurFields: true,
    exactOrigin: true,
  },
});
```

After connecting, navigate to the login page, wait for injection, then assert login success.

```ts
await page.goto("https://app.example.com/login", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(2_000);
await page.waitForURL(/dashboard|home/, { timeout: 30_000 });
```

## Track CAPTCHA solves

Enable solving only when the plan supports it.

```ts
const session = await client.sessions.create({
  solveCaptcha: true,
  timeout: 30 * 60 * 1000,
});
```

Use `sessions.captchas.status` to monitor progress.

```ts
type CaptchaTask = { status?: string };
type CaptchaState = {
  url?: string;
  isSolvingCaptcha?: boolean;
  tasks?: CaptchaTask[];
};

const activeStatuses = new Set(["detected", "validating", "solving"]);
const failedStatuses = new Set(["failed_to_detect", "failed_to_solve", "validation_failed"]);

async function waitForCaptchaSolution(sessionId: string, timeoutMs = 90_000) {
  const startedAt = Date.now();

  while (Date.now() - startedAt < timeoutMs) {
    const states = (await client.sessions.captchas.status(sessionId)) as CaptchaState[];
    const tasks = states.flatMap((state) => state.tasks ?? []);
    const failed = tasks.filter((task) => failedStatuses.has(task.status ?? ""));
    const active = states.filter(
      (state) =>
        state.isSolvingCaptcha ||
        (state.tasks ?? []).some((task) => activeStatuses.has(task.status ?? "")),
    );

    console.log({
      pages: states.length,
      activePages: active.length,
      solvedTasks: tasks.filter((task) => task.status === "solved").length,
      failedTasks: failed.length,
    });

    if (failed.length > 0) throw new Error("CAPTCHA solve failed");
    if (active.length === 0) return states;

    await new Promise((resolve) => setTimeout(resolve, 1_000));
  }

  throw new Error("Timed out waiting for CAPTCHA solving");
}
```

Call it after navigating to pages that may trigger CAPTCHA challenges:

```ts
await page.goto("https://example.com/protected", { waitUntil: "domcontentloaded" });
await waitForCaptchaSolution(session.id);
```

## Stealth and proxies

Start simple. Add stealth features only when needed and plan-compatible.

```ts
const session = await client.sessions.create({
  useProxy: true,
  solveCaptcha: true,
  timeout: 30 * 60 * 1000,
});
```

Use broad geotargeting before narrow city-level targeting.

```ts
const session = await client.sessions.create({
  useProxy: {
    geolocation: { country: "US" },
  },
});
```

Use BYOP when the user provides their own proxy server or the plan does not include Steel-managed proxies.

```ts
const session = await client.sessions.create({
  useProxy: {
    server: process.env.PROXY_SERVER!,
  },
});
```

Recommendations:

- Establish a baseline without proxies first
- Reuse profiles for sites where cookies and reputation help
- Add natural delays instead of rapid repeated actions
- Retry transient proxy errors such as `ERR_TUNNEL_CONNECTION_FAILED`
- Prefer country-level targeting unless location precision is required

## Bigger TypeScript frameworks

Use direct Playwright or Puppeteer for deterministic tools. Use Stagehand when the user wants natural-language browser actions like `act`, `extract`, and `observe`.

- Stagehand integration: https://docs.steel.dev/integrations/stagehand
- Stagehand recipe: https://docs.steel.dev/cookbook/stagehand
- Puppeteer integration: https://docs.steel.dev/integrations/puppeteer
- Playwright integration: https://docs.steel.dev/integrations/playwright
