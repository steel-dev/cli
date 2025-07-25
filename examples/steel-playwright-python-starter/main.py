import os
from typing import Optional
from dotenv import load_dotenv
from playwright.sync_api import sync_playwright
from steel import Steel

# Load environment variables from .env file
load_dotenv()

STEEL_API_KEY = os.getenv("STEEL_API_KEY")

STEEL_API_URL = os.getenv("STEEL_API_URL", "https://api.steel.dev")
STEEL_CONNECT_URL = os.getenv("STEEL_CONNECT_URL", "wss://connect.steel.dev")

STEEL_SESSION_ID = os.getenv("STEEL_SESSION_ID", None)

# Initialize Steel client with the API key from environment variables
client = Steel(
    steel_api_key=STEEL_API_KEY,
    base_url=STEEL_API_URL,
)


def main():
    session = None
    browser = None

    try:
        print("Creating Steel session...")

        params = {
            # === Basic Options ===
            # use_proxy=True,              # Use Steel's proxy network (residential IPs)
            # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
            # solve_captcha=True,          # Enable automatic CAPTCHA solving
            # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
            # === Browser Configuration ===
            # user_agent='custom-ua',      # Set a custom User-Agent
        }

        if STEEL_SESSION_ID:
            params["session_id"] = STEEL_SESSION_ID

        # Create a new Steel session with all available options
        session = client.sessions.create(**params)

        print(
            f"""\033[1;93mSteel Session created successfully!\033[0m
You can view the session live at \033[1;37m{session.session_viewer_url}\033[0m
        """
        )

        # Connect Playwright to the Steel session
        playwright = sync_playwright().start()

        cdp_url = f"{STEEL_CONNECT_URL}?apiKey={STEEL_API_KEY}&sessionId={session.id}"

        browser = playwright.chromium.connect_over_cdp(cdp_url)

        print("Connected to browser via Playwright")

        # Create page at existing context to ensure session is recorded.
        currentContext = browser.contexts[0]
        page = currentContext.new_page()

        # ============================================================
        # Your Automations Go Here!
        # ============================================================

        # Example script - Navigate to Hacker News and extract the top 5 stories
        print("Navigating to Hacker News...")
        page.goto("https://news.ycombinator.com", wait_until="load")

        # Find all story rows
        story_rows = page.locator("tr.athing").all()[:5]  # Get first 5 stories

        # Extract the top 5 stories using Playwright's locators
        print("\nTop 5 Hacker News Stories:")
        for i, row in enumerate(story_rows, 1):
            # Get the title and link from the story row
            title_element = row.locator(".titleline > a")
            title = title_element.text_content()
            link = title_element.get_attribute("href")

            # Get points from the following row
            points_element = row.locator("xpath=following-sibling::tr[1]").locator(
                ".score"
            )
            points = (
                points_element.text_content().split()[0]
                if points_element.count() > 0
                else "0"
            )

            # Print the story details
            print(f"\n{i}. {title}")
            print(f"   Link: {link}")
            print(f"   Points: {points}")

        # ============================================================
        # End of Automations
        # ============================================================

    except Exception as e:
        print(f"An error occurred: {e}")
    finally:
        # Cleanup: Gracefully close browser and release session when done
        if browser:
            browser.close()
            print("Browser closed")

        if session:
            print("Releasing session...")
            client.sessions.release(session.id)
            print("Session released")

        print("Done!")


# Run the script
if __name__ == "__main__":
    main()
