#!/usr/bin/env node

import React, {ReactElement} from 'react';
import {Box} from 'ink';
import fs from 'fs/promises';
import * as http from 'http';
import * as url from 'url';
import * as crypto from 'crypto';
import type {AddressInfo} from 'net';
import open from 'open';
import Callout from '../components/callout.js';
import UpdateNotification from '../components/updatenotification.js';
import {
	LOGIN_URL,
	SUCCESS_URL,
	TARGET_API_PATH,
	CONFIG_DIR,
	CONFIG_PATH,
	// SUCCESS_HTML,
} from '../utils/constants.js';
import {getApiKey} from '../utils/session.js';

type AuthState = {
	status: 'idle' | 'pending' | 'success' | 'error';
	message: string;
	apiKey?: string;
};

const LOGIN_TIMEOUT = 5 * 60 * 1000; // 5 minutes

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
					message: `You are already logged in`,
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
			<UpdateNotification />
			{state.status === 'pending' && (
				<Callout variant="info" title="Authentication in Progress">
					{state.message}
				</Callout>
			)}
			{state.status === 'success' && (
				<Callout variant="success" title="Authentication Successful">
					{state.message}
				</Callout>
			)}
			{state.status === 'error' && (
				<Callout variant="failed" title="Authentication Failed">
					{state.message}
				</Callout>
			)}
			{state.status === 'idle' && (
				<Callout variant="info" title="Starting Authentication">
					{state.message}
				</Callout>
			)}
		</Box>
	);
}

async function loginFlow(): Promise<{
	apiKey: string | null;
	name: string | null;
} | null> {
	return new Promise((resolve, reject) => {
		const state = crypto.randomBytes(16).toString('hex');

		const server = http.createServer(async (req, res) => {
			const {query} = url.parse(req.url ?? '', true);

			if (query['state'] !== state) {
				res.writeHead(400, {'Content-Type': 'text/plain'});
				res.end('Error: Invalid state parameter. Authentication failed.');
				reject(new Error('Invalid state parameter. Possible CSRF attack.'));
				return;
			}

			const jwt = query['jwt'] as string;
			if (!jwt) {
				res.writeHead(400, {'Content-Type': 'text/plain'});
				res.end('Error: JWT not found in callback.');
				reject(new Error('Callback did not include a JWT.'));
				return;
			}

			res.writeHead(302, {Location: SUCCESS_URL});
			res.end();

			clearTimeout(timeout);
			server.close();

			try {
				const auth = await createApiKeyUsingJWT(jwt);
				resolve(auth);
			} catch (error) {
				reject(error);
			}
		});

		const timeout = setTimeout(() => {
			server?.close();
			reject(new Error('Login timed out. Please try again.'));
		}, LOGIN_TIMEOUT);

		server.listen(0, '127.0.0.1', async () => {
			const {port} = server.address() as AddressInfo;

			const authUrl = new URL(LOGIN_URL);
			authUrl.searchParams.set('cli_redirect', 'true');
			authUrl.searchParams.set('port', port.toString());
			authUrl.searchParams.set('state', state);

			console.log('Opening your browser for authentication...');
			console.log('If it does not open automatically, please click:');
			console.log(authUrl.toString());

			try {
				await open(authUrl.toString());
			} catch {
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

async function createApiKeyUsingJWT(jwt: string): Promise<{
	apiKey: string | null;
	name: string | null;
} | null> {
	const response = await fetch(TARGET_API_PATH, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${jwt}`,
		},
		body: JSON.stringify({
			name: 'CLI',
		}),
	});

	if (!response.ok) {
		throw new Error(
			`Failed to get API key: ${response.status} ${response.statusText}`,
		);
	}

	const data = await response.json();

	if (!data.key || !data.name) {
		throw new Error('No API key found in response');
	}
	return {
		apiKey: data.key,
		name: data.name,
	};
}

async function saveApiKey(apiKey: string, name: string): Promise<void> {
	try {
		// Ensure config directory exists
		await fs.mkdir(CONFIG_DIR, {recursive: true});

		// Read existing config or create a new one
		let config: Record<string, string> = {};
		try {
			const existingConfig = await fs.readFile(CONFIG_PATH, 'utf-8');
			config = JSON.parse(existingConfig);
		} catch {
			// File doesn't exist or isn't valid JSON, use empty object
		}

		// Update with new API key
		config['apiKey'] = apiKey;
		config['name'] = name;
		config['instance'] = 'cloud';

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
