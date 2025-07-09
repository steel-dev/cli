#!/usr/bin/env node
import React, {ReactElement} from 'react';
import {Box, Text} from 'ink';
import fs from 'fs/promises';
import * as http from 'http';
import * as url from 'url';
import * as crypto from 'crypto';
import type {AddressInfo} from 'net';
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

export const alias = 'auth';

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

export function loginFlow() {
	return new Promise<string>((resolve, reject) => {
		const state = crypto.randomBytes(16).toString('hex');
		let server: http.Server;

		const timeout = setTimeout(() => {
			server?.close();
			reject(new Error('Login timed out. Please try again.'));
		}, LOGIN_TIMEOUT);

		server = http.createServer((req, res) => {
			const {query} = url.parse(req.url ?? '', true);

			if (query.state !== state) {
				res.writeHead(400, {'Content-Type': 'text/plain'});
				res.end('Error: Invalid state parameter. Authentication failed.');
				reject(new Error('Invalid state parameter. Possible CSRF attack.'));
				return;
			}

			const jwt = query.jwt as string;
			if (!jwt) {
				res.writeHead(400, {'Content-Type': 'text/plain'});
				res.end('Error: JWT not found in callback.');
				reject(new Error('Callback did not include a JWT.'));
				return;
			}

			res.writeHead(200, {'Content-Type': 'text/html'});
			res.end(successHtml);

			clearTimeout(timeout);
			server.close(() => {
				console.log('Local callback server shut down.');
			});
			resolve(jwt);
		});

		server.listen(0, '127.0.0.1', async () => {
			const {port} = server.address() as AddressInfo;
			console.log(`Local callback server listening on port ${port}...`);

			const authUrl = new URL(LOGIN_URL);
			authUrl.searchParams.set('cli_redirect', 'true');
			authUrl.searchParams.set('port', port.toString());
			authUrl.searchParams.set('state', state);

			console.log('Opening your browser for authentication...');
			console.log('If it does not open automatically, please click:');
			console.log(authUrl.toString());

			try {
				await open(authUrl.toString());
			} catch (error) {
				server.close();
				clearTimeout(timeout);
				reject(new Error('Failed to open browser'));
			}
		});

		server.on('error', err => {
			clearTimeout(timeout);
			reject(err);
		});
	});
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
