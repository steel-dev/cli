import puppeteer from 'puppeteer-core';

async function main() {
	let browser;

	try {
		// Connect Puppeteer to the Steel session
		browser = await puppeteer.launch();

		console.log('Connected to browser via Puppeteer');

		// Create a new page
		const page = await browser.newPage();

		// ============================================================
		// Your Automations Go Here!
		// ============================================================

		// Example script - Navigate to Hacker News and extract the top 5 stories (you can delete this)
		// Navigate to Hacker News
		console.log('Navigating to Hacker News...');
		await page.goto('https://news.ycombinator.com', {
			waitUntil: 'networkidle0',
		});

		// Extract the top 5 stories
		const stories = await page.evaluate(() => {
			const items = [];
			// Get all story items
			const storyRows = document.querySelectorAll('tr.athing');

			// Loop through first 5 stories
			for (let i = 0; i < 5; i++) {
				const row = storyRows[i];
				const titleElement = row.querySelector('.titleline > a');
				const subtext = row.nextElementSibling;
				const score = subtext?.querySelector('.score');

				// items.push({
				// 	title: titleElement?.textContent || '',
				// 	link: titleElement?.getAttribute('href') || '',
				// 	points: score?.textContent?.split(' ')[0] || '0',
				// });
			}
			return items;
		});

		// Print the results
		console.log('\nTop 5 Hacker News Stories:');
		stories.forEach((story, index) => {
			console.log(`\n${index + 1}. ${story.title}`);
			console.log(`   Link: ${story.link}`);
			console.log(`   Points: ${story.points}`);
		});

		// ============================================================
		// End of Automations
		// ============================================================
	} catch (error) {
		console.error('An error occurred:', error);
	} finally {
		// Cleanup: Gracefully close browser and release session when done (even when an error occurs)
		if (browser) {
			await browser.close();
			console.log('Browser closed');
		}

		console.log('Done!');
	}
}

// Run the script
main();
