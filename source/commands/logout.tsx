#!/usr/bin/env node
import {Box, Text} from 'ink';
import React, {ReactElement} from 'react';
import fs from 'fs/promises';
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
			<Text>
				{state.status === 'pending' && '⏳ '}
				{state.status === 'success' && '✅ '}
				{state.status === 'error' && '🚫 '}
				{state.message}
			</Text>
		</Box>
	);
}

async function logoutFlow(): Promise<boolean> {
	try {
		await fs.writeFile(CONFIG_PATH, JSON.stringify({}, null, 2));
		return true;
	} catch (error) {
		return false;
	}
}
