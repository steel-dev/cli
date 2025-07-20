#!/usr/bin/env node

import {Box} from 'ink';
import React, {ReactElement} from 'react';
import fs from 'fs/promises';
import Callout from '../components/callout.js';
import {CONFIG_PATH} from '../utils/constants.js';

type AuthState = {
	status: 'idle' | 'pending' | 'success' | 'error';
	message: string;
	apiKey?: string;
};

export const description = 'Logout from Steel CLI';

export default function Logout(): ReactElement {
	const [state, setState] = React.useState<AuthState>({
		status: 'idle',
		message: 'Starting authentication...',
	});

	React.useEffect(() => {
		const logout = async () => {
			setState({
				status: 'pending',
				message: 'Clearing API keys...',
			});

			try {
				const success = await logoutFlow();

				if (success) {
					setState({
						status: 'success',
						message: 'Successfully logged out. Have a great day!',
					});
				} else {
					setState({
						status: 'error',
						message: 'Logout failed or timed out.',
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

		logout();
	}, []);

	return (
		<Box flexDirection="column">
			{state.status === 'pending' && (
				<Callout variant="info" title="Logout in Progress">
					{state.message}
				</Callout>
			)}
			{state.status === 'success' && (
				<Callout variant="success" title="Logout Successful">
					{state.message}
				</Callout>
			)}
			{state.status === 'error' && (
				<Callout variant="failed" title="Logout Failed">
					{state.message}
				</Callout>
			)}
			{state.status === 'idle' && (
				<Callout variant="info" title="Starting Logout">
					{state.message}
				</Callout>
			)}
		</Box>
	);
}

async function logoutFlow(): Promise<boolean> {
	try {
		// Read the existing configuration
		const configData = await fs.readFile(CONFIG_PATH, 'utf-8');
		const config = JSON.parse(configData);

		// Remove the keys 'apiKey' and 'name'
		delete config.apiKey;
		delete config.name;

		// Write the updated configuration back to the file
		await fs.writeFile(CONFIG_PATH, JSON.stringify(config, null, 2));
		return true;
	} catch {
		return false;
	}
}
