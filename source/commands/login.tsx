#!/usr/bin/env node
import React, {ReactElement} from 'react';
import {Box, Text} from 'ink';
import fs from 'fs/promises';
import puppeteer from 'puppeteer';
import {
	TARGET_SITE,
	TARGET_API_PATH,
	CONFIG_DIR,
	CONFIG_PATH,
} from '../utils/constants.js';

import {getApiKey} from '../utils/session.js';

type AuthState = {
	status: 'idle' | 'pending' | 'success' | 'error';
	message: string;
	apiKey?: string;
};

export const description = 'Login to Steel CLI';

export default function Login(): ReactElement {
	const [state, setState] = React.useState<AuthState>({
		status: 'idle',
		message: 'Starting authentication...',
	});

	React.useEffect(() => {
		const login = async () => {
			const config = getApiKey();
			if (config) {
				setState({
					status: 'success',
					message: `You are already logged in with API Key: ${config.name}`,
				});
				return;
			}

			setState({
				status: 'pending',
				message: 'Launching browser for authentication...',
			});

			try {
				const auth = await loginFlow();

				if (auth && auth.apiKey && auth.name) {
					await saveApiKey(auth.apiKey, auth.name);

					setState({
						status: 'success',
						message: 'Authentication successful! Your API key has been saved.',
						apiKey: auth.apiKey,
					});
				} else {
					setState({
						status: 'error',
						message:
							'Authentication failed or timed out. Could not capture API key.',
					});
				}
			} catch (error) {
				setState({
					status: 'error',
					message: `Authentication error: ${
						error instanceof Error ? error.message : 'Unknown error'
					}`,
				});
			}
		};

		login();
	}, []);

	return (
		<Box flexDirection="column">
			<Text>
				{state.status === 'pending' && '⏳ '}
				{state.status === 'success' && '✅ '}
				{state.status === 'error' && '🚫 '}
				{state.message}
			</Text>
		</Box>
	);
}

async function loginFlow(): Promise<{
	apiKey: string | null;
	name: string | null;
} | null> {
	try {
		const browser = await puppeteer.launch({
			headless: false,
			defaultViewport: null,
			args: ['--no-sandbox', '--window-size=800,800'],
		});

		const page = (await browser.pages())[0];

		// Set up request interception
		let apiKey: string | null = null;
		let name: string | null = null;

		if (page) {
			// Listen for responses
			page.on('response', async response => {
				const url = response.url();

				// If they get put on the playground site, redirect them to the quickstart page
				if (url.includes('/playground')) {
					await page.goto('https://app.steel.dev/quickstart');
				}

				if (url.includes(TARGET_API_PATH) && response.status() === 200) {
					// Check if this is the API key endpoint we're looking for
					try {
						const responseBody = await response.json();
						// The structure of the response will depend on the API
						if (responseBody.key && responseBody.name) {
							apiKey = responseBody.key;
							name = responseBody.name;
						}

						// If we got the API key, close the browser
						if (apiKey && name) {
							await browser.close();
						}
					} catch (e) {
						// Response might not be JSON or might have another format
						console.error('Error parsing response:', e);
					}
				}
			});

			// Navigate to the login page
			await page.goto(TARGET_SITE);

			// Wait for up to 5 minutes for the user to log in and for us to capture the API key
			const timeout = 5 * 60 * 1000; // 5 minutes in milliseconds
			const startTime = Date.now();

			while (!apiKey && Date.now() - startTime < timeout) {
				await new Promise(resolve => setTimeout(resolve, 1000)); // Check every second
			}

			// Close the browser if it's still open
			if (browser.connected) {
				await browser.close();
			}
		}

		return {apiKey, name};
	} catch (error) {
		console.error('Error during Puppeteer session:', error);
		return null;
	}
}

async function saveApiKey(apiKey: string, name: string): Promise<void> {
	try {
		// Ensure config directory exists
		await fs.mkdir(CONFIG_DIR, {recursive: true});

		// Read existing config or create a new one
		let config: Record<string, any> = {};
		try {
			const existingConfig = await fs.readFile(CONFIG_PATH, 'utf-8');
			config = JSON.parse(existingConfig);
		} catch (error) {
			// File doesn't exist or isn't valid JSON, use empty object
		}

		// Update with new API key
		config['apiKey'] = apiKey;
		config['name'] = name;

		// Save the updated config
		await fs.writeFile(CONFIG_PATH, JSON.stringify(config, null, 2));
	} catch (error) {
		throw new Error(
			`Failed to save API key: ${
				error instanceof Error ? error.message : 'Unknown error'
			}`,
		);
	}
}
