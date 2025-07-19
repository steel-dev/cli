import React, {useState, useEffect} from 'react';
import {Box, Text, useApp, useInput} from 'ink';
import fs from 'fs/promises';
import path from 'path';
import {fileURLToPath} from 'url';
import {dirname} from 'path';
import CommandList from './commandlist.js';
import AnimatedLogo from './animatedlogo.js';

type Props = {
	command?: string;
};

// Function to scan command directory and get all available commands
async function getAvailableCommands(baseDir = 'commands') {
	try {
		const commandsPath = path.join(process.cwd(), 'dist', baseDir);

		// Check if directory exists first
		try {
			await fs.access(commandsPath);
		} catch {
			return [];
		}

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
				const commandPath = path.join(commandsPath, file.name);
				const commandModule = await import(`file://${commandPath}`);

				commands.push({
					name: file.name.replace(/\.js$/, ''),
					description: commandModule.description || 'No description available',
					isDefault: !!commandModule.isDefault,
					options: commandModule.options,
					args: commandModule.args,
				});
			}
		}
		return commands;
	} catch {
		return [];
	}
}

// Function to get command module for a specific command
async function getCommandModule(commandPath: string) {
	// Get the current file path to determine the project root
	const __filename = fileURLToPath(import.meta.url);
	const __dirname = dirname(__filename);
	const projectRoot = path.resolve(__dirname, '../../');

	// Try to load command directly
	const fullPath = path.join(projectRoot, 'dist/commands', `${commandPath}.js`);

	try {
		const commandModule = await import(`file://${fullPath}`);
		return commandModule;
	} catch (directErr) {
		// Try to handle subcommands (e.g. "browser start")
		const parts = commandPath.split(' ');
		if (parts.length > 1) {
			const subcommandPath = path.join(
				projectRoot,
				'dist/commands',
				...parts.slice(0, -1),
				`${parts[parts.length - 1]}.js`,
			);
			const commandModule = await import(`file://${subcommandPath}`);
			return commandModule;
		}
		throw directErr;
	}
}

// Function to extract option information in a formatted way
function extractOptionInfo(optionSchema: any) {
	if (!optionSchema) return [];

	try {
		const shape = optionSchema._def?.shape;
		if (!shape) return [];

		return Object.entries(shape).map(([name, def]: [string, any]) => {
			const description = def._def?.description || '';
			let type = def._def?.typeName || '';

			// Extract alias if available (in the description using option helper)
			let alias = '';
			if (description.includes('alias')) {
				try {
					// Try to parse the option description for aliases
					const aliasMatch = description.match(
						/alias['"]?\s*:\s*['"]([a-zA-Z0-9])['"]?/,
					);
					if (aliasMatch) {
						alias = aliasMatch[1];
					}
				} catch (e) {
					// Ignore parsing errors
				}
			}

			// Get readable description text
			let desc = description;
			if (desc.startsWith('{')) {
				try {
					const parsed = JSON.parse(desc);
					desc = parsed.description || '';
				} catch (e) {
					// If parsing fails, use the original description
				}
			}

			// Check if option is optional
			const isOptional = def._def?.typeName === 'ZodOptional';

			return {
				name,
				description: desc,
				alias,
				type,
				isOptional,
			};
		});
	} catch (err) {
		console.error('Error extracting option info:', err);
		return [];
	}
}

// Function to extract argument information
function extractArgumentInfo(argsSchema: any, argsLabels: string[]) {
	if (!argsSchema) return [];

	try {
		// Handle tuple-style arguments
		if (argsSchema._def?.typeName === 'ZodTuple') {
			const items = argsSchema._def.items || [];
			return items.map((item: any, index: number) => {
				const description = item._def?.description || '';
				const isOptional = item._def?.typeName === 'ZodOptional';

				// Parse name and description from the description string
				let name = argsLabels[index] || `args${index + 1}`;
				let desc = description;

				if (description.startsWith('{')) {
					const parsed = JSON.parse(description);
					name = parsed.name || name;
					desc = parsed.description || '';
				}

				return {
					name,
					description: desc,
					isOptional,
					index,
				};
			});
		}

		// Handle array-style arguments (variable number of args)
		if (argsSchema._def?.typeName === 'ZodArray') {
			const itemType = argsSchema._def?.type;
			const description = itemType?._def?.description || '';

			// Parse name and description
			let name = 'args';
			let desc = description;

			if (description.startsWith('{')) {
				try {
					const parsed = JSON.parse(description);
					name = parsed.name || name;
					desc = parsed.description || '';
				} catch {
					// If parsing fails, use the original description
				}
			}

			return [
				{
					name,
					description: desc,
					isArray: true,
				},
			];
		}

		return [];
	} catch {
		return [];
	}
}

export default function Help({command}: Props) {
	const {exit} = useApp();
	const [commands, setCommands] = useState([]);
	const [error, setError] = useState('');
	const [commandInfo, setCommandInfo] = useState(null);

	useInput(() => {
		exit();
	});

	// Load commands and potentially specific command help
	useEffect(() => {
		async function load() {
			try {
				// Load all commands for the main help menu
				const cmds = await getAvailableCommands();
				setCommands(cmds);

				// If a specific command was requested, get its module info
				if (command) {
					try {
						const commandModule = await getCommandModule(command);

						if (commandModule) {
							// Extract command information
							setCommandInfo({
								name: command,
								description: commandModule.description || '',
								options: extractOptionInfo(commandModule.options),
								args: extractArgumentInfo(
									commandModule.args,
									commandModule.argsLabels,
								),
								usage: `steel ${command}${commandModule.args ? ' [arguments]' : ''}${commandModule.options ? ' [options]' : ''}`,
							});
						} else {
							setError(`Command '${command}' not found.`);
						}
					} catch {
						// If command doesn't exist or fails
						setError(
							`Command '${command}' not found or help is not available.`,
						);
					}
				}
			} catch {
				// Handle error
				setError('Failed to load help information.');
			}
		}

		// Execute load and handle any uncaught errors
		load();
	}, [command, exit]);

	function Sidebar() {
		return (
			<Box flexDirection="column" justifyContent="center" alignItems="center">
				<Box marginBottom={1} alignItems="center">
					<Text bold color="blue">
						Steel
					</Text>
					<Text> - Open-source browser API for AI agents</Text>
				</Box>
				<AnimatedLogo />
				<Box marginTop={1} flexDirection="column" alignItems="center">
					<Text bold>DOCUMENTATION</Text>
					<Text>https://docs.steel.dev</Text>
				</Box>
				<Box marginTop={1} flexDirection="column" alignItems="center">
					<Text bold>GITHUB</Text>
					<Text>https://github.com/steel-dev/steel-browser</Text>
				</Box>
			</Box>
		);
	}

	// Show specific command help
	if (command) {
		return (
			<Box flexDirection="row" justifyContent="space-evenly">
				<Sidebar />
				<Box flexDirection="column">
					<Box marginBottom={1}>
						<Text bold color="blue">
							Steel CLI
						</Text>
						<Text> - Help for command: </Text>
						<Text color="green">{command}</Text>
					</Box>

					{error ? (
						<Box marginBottom={1}>
							<Text color="red">{error}</Text>
						</Box>
					) : commandInfo ? (
						<>
							{commandInfo.description && (
								<Box marginBottom={1}>
									<Text>{commandInfo.description}</Text>
								</Box>
							)}

							<Box marginBottom={1}>
								<Text bold>USAGE</Text>
							</Box>
							<Box marginBottom={1} paddingLeft={2}>
								<Text>$ {commandInfo.usage}</Text>
							</Box>

							{commandInfo.args && commandInfo.args.length > 0 && (
								<>
									<Box marginBottom={1}>
										<Text bold>ARGUMENTS</Text>
									</Box>
									{commandInfo.args.map((arg, index) => (
										<Box key={index} paddingLeft={2} marginBottom={0}>
											<Box width={20}>
												<Text color="yellow">{arg.name}</Text>
												{arg.isOptional && (
													<Text color="gray"> (optional)</Text>
												)}
											</Box>
											<Text>{arg.description}</Text>
										</Box>
									))}
								</>
							)}

							{commandInfo.options && commandInfo.options.length > 0 && (
								<>
									<Box marginBottom={1} marginTop={1}>
										<Text bold>OPTIONS</Text>
									</Box>
									{commandInfo.options.map((opt, index) => (
										<Box key={index} paddingLeft={2} marginBottom={0}>
											<Box width={20}>
												{opt.alias && (
													<>
														<Text>-{opt.alias}, </Text>
													</>
												)}
												<Text>--{opt.name}</Text>
											</Box>
											<Text>{opt.description}</Text>
										</Box>
									))}
								</>
							)}
						</>
					) : (
						<Box marginBottom={1} flexDirection="column">
							<Text>No help information available for this command.</Text>
						</Box>
					)}

					<Box marginTop={1}>
						<Text dimColor>For the main help menu, run:</Text>
					</Box>
					<Box marginBottom={1} paddingLeft={2}>
						<Text>$ steel</Text>
					</Box>
				</Box>
			</Box>
		);
	}

	// Main help screen
	return (
		<Box flexDirection="row" justifyContent="space-evenly">
			<Sidebar />
			<Box flexDirection="column">
				<Box>
					<Text bold>USAGE</Text>
				</Box>
				<Box marginBottom={1} paddingLeft={2}>
					<Text>$ steel [command] [options]</Text>
				</Box>

				<Box>
					<Text bold>COMMANDS</Text>
				</Box>
				{commands.length > 0 ? (
					<CommandList commands={commands} />
				) : (
					<Box paddingLeft={2}>
						<Text color="yellow">No commands found. Try using 'steel'.</Text>
					</Box>
				)}

				<Box marginTop={1}>
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
			</Box>
		</Box>
	);
}
