"""
Steel Browser Use Starter Template
Integrates Steel with browser-use framework to create an AI agent for web interactions.
Requires STEEL_API_KEY & OPENAI_API_KEY in .env file.
"""

import os
import asyncio
from dotenv import load_dotenv
from steel import Steel
from browser_use import Agent, BrowserSession
from browser_use.llm import ChatOpenAI
from browser_use import Agent

# 1. Initialize environment and clients
load_dotenv()

# Get API keys
STEEL_API_KEY = os.getenv("STEEL_API_KEY")
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")

STEEL_API_URL = os.getenv("STEEL_API_URL", "https://api.steel.dev")
STEEL_CONNECT_URL = os.getenv("STEEL_CONNECT_URL", "wss://connect.steel.dev")

STEEL_SESSION_ID = os.getenv('STEEL_SESSION_ID', "")
# The agent's main instructions
TASK = os.getenv("TASK", "Go to https://docs.steel.dev/, open the changelog, and tell me what's new.")

if not STEEL_API_KEY or not OPENAI_API_KEY:
    raise ValueError("STEEL_API_KEY and OPENAI_API_KEY must be set in .env file")

# 2. Initialize Steel client and create session
client = Steel(steel_api_key=STEEL_API_KEY, base_url=STEEL_API_URL)


# 3. Create a Steel session to get a remote browser instance for your browser-use agent.
print("Creating Steel session...")
session = client.sessions.create(
	session_id=STEEL_SESSION_ID, # Optional session ID
)
print(f"Session created at {session.session_viewer_url}")

print(
    f"\033[1;93mSteel Session created!\033[0m\n"
    f"View session at \033[1;37m{session.session_viewer_url}\033[0m\n"
)

# 4. Connect browser-use to Steel
# Replace YOUR_STEEL_API_KEY with your actual API key
cdp_url = f"{STEEL_CONNECT_URL}?apiKey={STEEL_API_KEY}&sessionId={session.id}"

# 5. Create and configure the AI agent
model = ChatOpenAI(model="gpt-4o", temperature=0.3, api_key=OPENAI_API_KEY)

agent = Agent(task=TASK, llm=model, browser_session=BrowserSession(cdp_url=cdp_url))

# 6. Run the agent and handle cleanup

async def main():
    try:
        await agent.run()
    except Exception as e:
        print(f"Error: {e}")
    finally:
        # Clean up resources
        if session:
            client.sessions.release(session.id)
            print("Session released")
        print("Done!")


if __name__ == "__main__":
    asyncio.run(main())
