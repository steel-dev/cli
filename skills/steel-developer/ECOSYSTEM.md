# Steel Ecosystem

Use this file to route users to the right Steel-supported integration. Keep direct Playwright and Puppeteer as the baseline unless a bigger framework clearly matches the user's goal.

## Routing Principles

- Use direct Playwright or Puppeteer for deterministic scripts, backend jobs, tests, and workflows with stable selectors.
- Use Stagehand when a TypeScript user wants natural-language browser actions like `act`, `extract`, and `observe`.
- Use Browser Use when a Python user wants an LLM browser agent loop.
- Use computer-use integrations when the model should reason from screenshots and emit low-level actions.
- Use typed agent frameworks when the user is building a product with tools, typed outputs, state, tracing, or multi-agent orchestration.
- Use the `steel-browser` skill when the agent should browse, extract, screenshot, or fill forms now instead of writing reusable code.
- Ignore Selenium unless the user explicitly asks for it.

## Source Of Truth

- Integrations index: https://docs.steel.dev/integrations
- Cookbook index: https://docs.steel.dev/cookbook
- API reference: https://steel.apidocumentation.com/api-reference
- Node SDK: https://github.com/steel-dev/steel-node
- Python SDK: https://github.com/steel-dev/steel-python

When fetching integration docs with `curl`, use `curl -sSfL` and the `/llms.mdx/<page-path>` URL form.

## Choose The Integration

| User goal | Recommended route | Language | Docs |
| --- | --- | --- | --- |
| Deterministic browser script | Playwright | TypeScript or Python | https://docs.steel.dev/integrations/playwright |
| Deterministic Chrome script | Puppeteer | TypeScript | https://docs.steel.dev/integrations/puppeteer |
| Natural-language browser actions | Stagehand | TypeScript | https://docs.steel.dev/integrations/stagehand |
| LLM browser agent loop | Browser Use | Python | https://docs.steel.dev/integrations/browser-use |
| Typed browser tools and handoffs | OpenAI Agents SDK | TypeScript or Python | https://docs.steel.dev/integrations/openai-agents-sdk |
| Typed streaming agent product | Vercel AI SDK | TypeScript | https://docs.steel.dev/integrations/ai-sdk |
| TypeScript agent registry/studio | Mastra | TypeScript | https://docs.steel.dev/integrations/mastra |
| Python state-machine agent | LangGraph | Python | https://docs.steel.dev/integrations/langgraph |
| Provider-agnostic typed Python agent | Pydantic AI | Python | https://docs.steel.dev/integrations/pydantic-ai |
| Python multi-agent team | Agno or CrewAI | Python | https://docs.steel.dev/integrations/agno |
| TypeScript multi-agent network | AgentKit | TypeScript | https://docs.steel.dev/integrations/agentkit |
| Model-native screenshot/action loop | Claude, OpenAI, or Gemini Computer Use | TypeScript or Python | https://docs.steel.dev/integrations/claude-computer-use |
| Reliable structured browser agent | Notte or Magnitude | Python or TypeScript | https://docs.steel.dev/integrations/notte |
| Coding agent should operate browser now | `steel-browser` skill | CLI | https://docs.steel.dev/overview/steel-cli |

## Baseline: Playwright And Puppeteer

Use these first for most reusable workflows.

Rules:

- Create a Steel session in application code.
- Construct `wss://connect.steel.dev?apiKey=...&sessionId=...` explicitly.
- Connect over CDP.
- Reuse the default context and page.
- Release the Steel session in cleanup.
- Add profiles, credentials, files, extensions, proxies, and CAPTCHA handling only as needed.

Cookbook:

- Playwright: https://docs.steel.dev/cookbook/playwright
- Puppeteer: https://docs.steel.dev/cookbook/puppeteer

## Stagehand

Use Stagehand when TypeScript users want natural-language actions and structured extraction without hand-writing every selector.

Good fit:

- Sites with changing UI where semantic actions are easier than selectors.
- Extracting structured data from pages.
- Prototypes that may later be hardened with direct Playwright locators.

Avoid when:

- The workflow is stable and deterministic enough for direct Playwright.
- The user needs Python.

References:

- Integration: https://docs.steel.dev/integrations/stagehand
- Recipe: https://docs.steel.dev/cookbook/stagehand

## Browser Use

Use Browser Use when Python users want an autonomous browser agent that can navigate, fill forms, and extract data.

Good fit:

- Agentic tasks where the exact path is not known ahead of time.
- Python applications using vision-capable models.
- CAPTCHA-heavy flows where a Browser Use tool can call Steel CAPTCHA status/solve helpers.

Gotchas:

- Pass Steel's CDP URL into the Browser Use browser session.
- Keep Steel session lifecycle ownership explicit and release sessions in cleanup.
- For downloads and uploads, consult Files API guidance in [APIS.md](APIS.md).

References:

- Integration: https://docs.steel.dev/integrations/browser-use
- Recipe: https://docs.steel.dev/cookbook/browser-use
- CAPTCHA auto recipe: https://docs.steel.dev/cookbook/browser-use-captcha-auto
- CAPTCHA manual recipe: https://docs.steel.dev/cookbook/browser-use-captcha-manual

## Computer Use Integrations

Use computer-use integrations when the model should see screenshots and emit click/type/scroll actions rather than using DOM selectors.

Routes:

- Claude Computer Use: https://docs.steel.dev/integrations/claude-computer-use
- OpenAI Computer Use: https://docs.steel.dev/integrations/openai-computer-use
- Gemini Computer Use: https://docs.steel.dev/integrations/gemini-computer-use

Good fit:

- Visual workflows where DOM selectors are unreliable.
- Mobile or responsive UI tasks.
- Human-like interaction loops that need screenshots.

Gotchas:

- Use Steel session dimensions/mobile mode intentionally.
- Route model actions back through Steel's computer/session APIs.
- Preserve a viewer URL or trace for debugging.

## Typed Agent Frameworks

Use typed agent frameworks when building production agents with tools, tracing, streaming, structured output, state, or handoffs.

TypeScript routes:

- Vercel AI SDK for typed tools and streaming: https://docs.steel.dev/integrations/ai-sdk
- OpenAI Agents SDK for handoffs and guardrails: https://docs.steel.dev/integrations/openai-agents-sdk
- Mastra for TypeScript agents, registry, and local studio: https://docs.steel.dev/integrations/mastra
- AgentKit for TypeScript multi-agent networks: https://docs.steel.dev/integrations/agentkit

Python routes:

- LangGraph for explicit state-machine agents: https://docs.steel.dev/integrations/langgraph
- Pydantic AI for provider-agnostic typed agents: https://docs.steel.dev/integrations/pydantic-ai
- Agno for Python agent teams with memory/reasoning: https://docs.steel.dev/integrations/agno
- CrewAI for multi-agent crews: https://docs.steel.dev/integrations/crewai

Specialized browser-agent routes:

- Notte for reliable Python navigation and structured outputs: https://docs.steel.dev/integrations/notte
- Magnitude for TypeScript natural-language browser agents: https://docs.steel.dev/integrations/magnitude

Pattern:

- Expose Steel operations as framework tools.
- Keep session creation, CDP connection, and release explicit.
- Return `session.id` and `session.sessionViewerUrl` from open-session tools.
- Add a cleanup tool or cleanup path for long-running agents.
- Use agent traces and viewer URLs for debugging productized agents.

## Coding Agent CLI Integrations

Use CLI integrations when a coding agent should operate Steel itself from the terminal.

Routes:

- Claude Code: https://docs.steel.dev/integrations/claude-code
- Codex: https://docs.steel.dev/integrations/codex
- OpenClaw: https://docs.steel.dev/integrations/openclaw
- Pi Agent: https://docs.steel.dev/integrations/pi-agent

For reusable code, come back to `steel-developer`. For live browsing by the agent, use `steel-browser`.
