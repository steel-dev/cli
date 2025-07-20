#!/usr/bin/env node

import React from 'react';
import {Box, Text} from 'ink';
import Callout from '../components/callout.js';
import {getApiKey, getSettings} from '../utils/session.js';

export const description = 'Display information about the current session';

export default function Info() {
	const apiKey = getApiKey();
	const settings = getSettings();
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
					<Callout variant="warning" title="Authentication Required">
						You are not logged in. Please run `steel login` to authenticate.
					</Callout>
				)}
				{settings ? (
					<Box flexDirection="column">
						<Text>{`{`}</Text>
						{Object.keys(settings).map(key =>
							key === 'apiKey' ? (
								<Text key={key}>
									{'  '}
									<Text color="yellow">{key}:</Text>{' '}
									<Text color="cyan">
										{settings[key as keyof typeof settings]}
									</Text>
								</Text>
							) : (
								<Text key={key}>
									{'  '}
									<Text color="yellow">{key}:</Text>{' '}
									<Text color="cyan">
										{settings[key as keyof typeof settings]}
									</Text>
								</Text>
							),
						)}
						<Text>{`}`}</Text>
					</Box>
				) : null}
			</Box>
		</Box>
	);
}
