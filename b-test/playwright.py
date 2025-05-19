import os
from dotenv import load_dotenv
from playwright.sync_api import sync_playwright

# Load environment variables from .env file
load_dotenv()

def main():
    browser = None

    try:

        # Connect Playwright to the Steel session
        playwright = sync_playwright().start()


        # Create page at existing context to ensure session is recorded.
        currentContext = browser.contexts[0]
        page = currentContext.new_page()

        # ============================================================
        # Your Automations Go Here!
        # ============================================================

        # Example script - Navigate to Hacker News and extract the top 5 stories
        print("Navigating to Hacker News...")
        page.goto("https://news.ycombinator.com", wait_until="networkidle")

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
            points_element = row.locator("xpath=following-sibling::tr[1]").locator(".score")
            points = points_element.text_content().split()[0] if points_element.count() > 0 else "0"

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

        print("Done!")

# Run the script
if __name__ == "__main__":
    main()
