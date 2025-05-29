import React from 'react';
import {Box, Text} from 'ink';
import {getApiKey} from '../utils/session.js';

export const description = 'Display information about the current session';

export default function Info() {
	const apiKey = getApiKey();
	return (
		<Box borderStyle="bold" flexDirection="column">
			<Box marginLeft={1} flexDirection="column">
				{apiKey ? (
					<Box flexDirection="column">
						<Text>{`{`}</Text>
						{Object.keys(apiKey).map(key =>
							key === 'apiKey' ? (
								<Text key={key}>
									{'  '}
									<Text color="yellow">{key}:</Text>{' '}
									<Text color="cyan">
										{apiKey[key as keyof typeof apiKey].substring(0, 7) + '...'}
									</Text>
								</Text>
							) : (
								<Text key={key}>
									{'  '}
									<Text color="yellow">{key}:</Text>{' '}
									<Text color="cyan">{apiKey[key as keyof typeof apiKey]}</Text>
								</Text>
							),
						)}
						<Text>{`}`}</Text>
					</Box>
				) : (
					<Text color="red">You are not logged in 🫠</Text>
				)}
			</Box>
		</Box>
	);
}
