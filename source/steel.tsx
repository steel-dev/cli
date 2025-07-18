#!/usr/bin/env node
import Pastel from 'pastel';
import React from 'react';
import {Text, Box} from 'ink';
import Link from 'ink-link';
import {render} from 'ink';
// import figures from 'figures';

// Check if help flag is provided
const helpFlag = process.argv.includes('--help') || process.argv.includes('-h');
console.log('help', helpFlag);
const args = process.argv.slice(2).filter(arg => !arg.startsWith('-'));
const command = args.length > 0 ? args[0] : '';

// Custom help component that shows a branded help message
const CustomHelp = () => (
	<Box flexDirection="column">
		<Box marginBottom={1}>
			<Text bold color="blue">
				Steel CLI
			</Text>
			<Text> - Open-source browser API for AI agents</Text>
		</Box>

		<Box marginBottom={1}>
			<Text bold>USAGE</Text>
		</Box>
		<Box marginBottom={1} paddingLeft={2}>
			<Text>$ steel [command] [options]</Text>
		</Box>

		<Box marginBottom={1}>
			<Text bold>COMMON OPTIONS</Text>
		</Box>
		<Box paddingLeft={2} marginBottom={0}>
			<Box width={20}>
				<Text>-h, --help</Text>
			</Box>
			<Text>Display help for a command</Text>
		</Box>
		<Box paddingLeft={2} marginBottom={1}>
			<Box width={20}>
				<Text>-v, --version</Text>
			</Box>
			<Text>Display Steel CLI version</Text>
		</Box>

		<Box marginTop={1}>
			<Text bold>DOCUMENTATION</Text>
		</Box>
		<Box marginBottom={1} paddingLeft={2}>
			<Link url="https://docs.steel.dev">
				<Text color="cyan">https://docs.steel.dev</Text>
			</Link>
		</Box>

		<Box marginTop={1}>
			<Text bold>GITHUB</Text>
		</Box>
		<Box marginBottom={1} paddingLeft={2}>
			<Link url="https://github.com/steel-dev/steel-browser">
				<Text color="cyan">https://github.com/steel-dev/steel-browser</Text>
			</Link>
		</Box>

		<Box marginTop={1}>
			<Text dimColor>
				Steel is an open-source browser API purpose-built for AI agents.
			</Text>
		</Box>
		<Box marginTop={1}>
			<Text dimColor>
				Give one or 1,000 agents the ability to interact with any website.
			</Text>
		</Box>
	</Box>
);

// If help flag is provided and it's the main help (not for a specific command)
if (helpFlag && !command) {
	render(<CustomHelp />);
	process.exit(0);
}

const app = new Pastel({
	importMeta: import.meta,
	version: '0.0.1-alpha.7',
});

await app.run();
