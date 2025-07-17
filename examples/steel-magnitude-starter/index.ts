import { startBrowserAgent } from "magnitude-core";
import z from 'zod';
import Steel from 'steel-sdk/index.mjs';
import dotenv from 'dotenv';

dotenv.config();

const STEEL_API_KEY = process.env.STEEL_API_KEY;
const ANTHROPIC_API_KEY = process.env.ANTHROPIC_API_KEY;

const STEEL_API_URL = process.env.STEEL_API_URL || 'https://api.steel.dev';
const STEEL_CONNECT_URL =
	process.env.STEEL_CONNECT_URL || 'wss://connect.steel.dev';

const STEEL_SESSION_ID = process.env.STEEL_SESSION_ID || undefined;

const TASK = process.env.TASK || "Go to https://docs.steel.dev/, open the changelog, and tell me what's new.";

// Initialize Steel client with the API key from environment variables
const client = new Steel({
	steelAPIKey: STEEL_API_KEY,
	baseURL: STEEL_API_URL,
});

async function main() {
	let session;
	try {
		console.log('Creating Steel session...');

		// Create a new Steel session with all available options
		session = await client.sessions.create({
			// === Basic Options ===
			sessionId: STEEL_SESSION_ID, // Optional session ID
			// useProxy: true, // Use Steel's proxy network (residential IPs)
			// proxyUrl: 'http://...',         // Use your own proxy (format: protocol://username:password@host:port)
			// solveCaptcha: true,             // Enable automatic CAPTCHA solving
			// sessionTimeout: 1800000,        // Session timeout in ms (default: 5 mins)
			// === Browser Configuration ===
			// userAgent: 'custom-ua-string',  // Set a custom User-Agent
		});

		console.log(
			`\x1b[1;93mSteel Session created!\x1b[0m\n` +
			`View session at \x1b[1;37m${session.sessionViewerUrl}\x1b[0m`,
		);

		const cdpUrl = `${STEEL_CONNECT_URL}?apiKey=${STEEL_API_KEY}&sessionId=${session.id}`;

		const agent = await startBrowserAgent({
			// Starting URL for agent
			// url: 'https://docs.steel.dev/',
			// Show thoughts and actions
			narrate: true,
			browser: {
				cdp: cdpUrl,
			},
			// LLM configuration
			llm: {
				provider: 'anthropic',
				options: {
					model: 'claude-sonnet-4-20250514',
					apiKey: ANTHROPIC_API_KEY
				}
			},
		});

		// Magnitude can handle high-level tasks
		await agent.act(TASK);

		// Stop agent and browser
		await agent.stop();

	} catch (error) {
		console.error('An error occurred:', error);
	} finally {

		if (session) {
			console.log('Releasing session...');
			await client.sessions.release(session.id);
			console.log('Session released');
		}

		console.log('Done!');
	}
}

main();
