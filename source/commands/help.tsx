#!/usr/bin/env node

import React, {useState, useEffect} from 'react';
import {Box, Text} from 'ink';
import Link from 'ink-link';
import {exec} from 'child_process';
import {promisify} from 'util';
import zod from 'zod';
import {option} from 'pastel';
import fs from 'fs/promises';
import path from 'path';
import Spinner from 'ink-spinner';

export const description = 'Display help information about Steel CLI';
export const isDefault = false;
export const alias = 'h';

// Define command options
export const options = zod.object({
	command: zod.string().optional().describe('Command to get help for'),
});

const execAsync = promisify(exec);

type Props = {
	options: zod.infer<typeof options>;
};

// Function to scan command directory and get all available commands
async function getAvailableCommands(baseDir = 'commands') {
	try {
		const commandsPath = path.join(process.cwd(), 'dist', baseDir);
		const files = await fs.readdir(commandsPath, {withFileTypes: true});

		const commands = [];

		for (const file of files) {
			// Skip _app.tsx and helper files
			if (file.name.startsWith('_') || file.name === 'help.js') continue;

			if (file.isDirectory()) {
				// It's a subcommand directory
				const subcommands = await getAvailableCommands(
					`${baseDir}/${file.name}`,
				);
				commands.push(
					...subcommands.map(cmd => ({
						...cmd,
						name: `${file.name} ${cmd.name}`,
					})),
				);
			} else if (file.name.endsWith('.js')) {
				// It's a command file
				try {
					const commandPath = path.join(commandsPath, file.name);
					const commandModule = await import(`file://${commandPath}`);

					commands.push({
						name: file.name.replace(/\.js$/, ''),
						description:
							commandModule.description || 'No description available',
						isDefault: !!commandModule.isDefault,
					});
				} catch (err) {
					// Skip commands that can't be imported
				}
			}
		}

		return commands;
	} catch (err) {
		console.error('Error getting commands:', err);
		return [];
	}
}

export default function Help({options}: Props) {
	const [commands, setCommands] = useState([]);
	const [loading, setLoading] = useState(true);
	const [helpOutput, setHelpOutput] = useState('');
	const [error, setError] = useState('');

	// Load commands and potentially specific command help
	useEffect(() => {
		async function load() {
			try {
				// Load all commands for the main help menu
				const cmds = await getAvailableCommands();
				setCommands(cmds);

				// If a specific command was requested, get its help output
				if (options.command) {
					try {
						// Execute the command with --help to get the built-in help output
						const {stdout} = await execAsync(
							`node ${process.argv[1]} ${options.command} --help`,
						);
						setHelpOutput(stdout);
					} catch (err) {
						// If command doesn't exist or fails
						setError(
							`Command '${options.command}' not found or help is not available.`,
						);
					}
				}
			} catch (err) {
				// Handle error
				setError('Failed to load help information.');
			} finally {
				setLoading(false);
			}
		}

		load();
	}, [options.command]);

	// Show loading state
	if (loading) {
		return (
			<Box>
				<Text>
					<Spinner type="dots" /> Loading help information...
				</Text>
			</Box>
		);
	}

	// Show specific command help
	if (options.command) {
		return (
			<Box flexDirection="column">
				<Box marginBottom={1}>
					<Text bold color="blue">
						Steel CLI
					</Text>
					<Text> - Help for command: </Text>
					<Text color="green">{options.command}</Text>
				</Box>

				{error ? (
					<Box marginBottom={1}>
						<Text color="red">{error}</Text>
					</Box>
				) : (
					<Box marginBottom={1} flexDirection="column">
						<Text>{helpOutput}</Text>
					</Box>
				)}

				<Box marginTop={1}>
					<Text dimColor>For the main help menu, run:</Text>
				</Box>
				<Box marginBottom={1} paddingLeft={2}>
					<Text>$ steel help</Text>
				</Box>
			</Box>
		);
	}

	// Main help screen
	return (
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
				<Text bold>COMMANDS</Text>
			</Box>
			{commands.map(command => (
				<Box key={command.name} paddingLeft={2} marginBottom={0}>
					<Box width={20}>
						<Text color="green">{command.name}</Text>
					</Box>
					<Text>{command.description}</Text>
				</Box>
			))}

			<Box paddingLeft={2} marginBottom={1}>
				<Box width={20}>
					<Text color="green">help [command]</Text>
				</Box>
				<Text>Display help information</Text>
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
				<Text dimColor>For detailed help on a specific command, run:</Text>
			</Box>
			<Box marginBottom={1} paddingLeft={2}>
				<Text>$ steel help [command]</Text>
			</Box>
		</Box>
	);
}
