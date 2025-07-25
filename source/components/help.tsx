import React, {useState, useEffect} from 'react';
import {Box, Text} from 'ink';
import fs from 'fs/promises';
import path from 'path';
import {fileURLToPath} from 'url';
import {dirname} from 'path';
import CommandList from './commandlist.js';
import CLIWelcomeMessage from './cliwelcomemessage.js';
import TemplateList from './templatelist.js';
import {
	isTemplateCommand,
	createTemplateCommandModule,
} from '../utils/templateHelp.js';
import {TEMPLATES} from '../utils/constants.js';

type Props = {
	command?: string;
};

interface CommandModule {
	description?: string;
	isDefault?: boolean;
	options?: unknown;
	args?: unknown;
	argsLabels?: string[];
	template?: unknown;
}

interface ZodDef {
	typeName?: string;
	description?: string;
	shape?: Record<string, ZodDef>;
	items?: ZodDef[];
	type?: ZodDef;
	_def?: ZodDef;
}

interface ZodSchema {
	_def?: ZodDef;
	shape?: Record<string, ZodSchema>;
}

// Function to scan command directory and get all available commands
async function getAvailableCommands(baseDir = 'commands') {
	try {
		const __filename = fileURLToPath(import.meta.url);
		const __dirname = dirname(__filename);
		const projectRoot = path.resolve(__dirname, '../../');
		const commandsPath = path.join(projectRoot, 'dist', baseDir);

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
				const commandModule = (await import(
					`file://${commandPath}`
				)) as CommandModule;

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
	// Check if this is a template-specific help request first
	const templateCommand = isTemplateCommand(commandPath);
	if (templateCommand) {
		// Return virtual command module for template
		return createTemplateCommandModule(
			templateCommand.command as 'run' | 'forge',
			templateCommand.template,
		);
	}

	// Get the current file path to determine the project root
	const __filename = fileURLToPath(import.meta.url);
	const __dirname = dirname(__filename);
	const projectRoot = path.resolve(__dirname, '../../');

	// Try to load command directly
	const fullPath = path.join(projectRoot, 'dist/commands', `${commandPath}.js`);

	try {
		const commandModule = (await import(`file://${fullPath}`)) as CommandModule;
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
			const commandModule = (await import(
				`file://${subcommandPath}`
			)) as CommandModule;
			return commandModule;
		}
		throw directErr;
	}
}

// Function to extract option information in a formatted way
function extractOptionInfo(optionSchema: ZodSchema) {
	if (!optionSchema) return [];

	try {
		const shape = optionSchema.shape || optionSchema._def?.shape;
		if (!shape) return [];

		return Object.entries(shape).map(([name, def]: [string, ZodSchema]) => {
			const description = def._def?.description || '';
			const type = def._def?.typeName || '';

			// Extract alias and description from pastel option config
			let alias = '';
			let desc = description;

			if (description.includes('__pastel_option_config__')) {
				try {
					const configMatch = description.match(/__pastel_option_config__(.+)/);
					if (configMatch) {
						const config = JSON.parse(configMatch[1]);
						desc = config.description || '';
						alias = config.alias || '';
					}
				} catch {
					// If parsing fails, use the original description
					desc = description;
				}
			} else if (description.startsWith('{')) {
				try {
					const parsed = JSON.parse(description);
					desc = parsed.description || '';
					alias = parsed.alias || '';
				} catch {
					// If parsing fails, use the original description
					desc = description;
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
function extractArgumentInfo(argsSchema: ZodSchema, argsLabels: string[]) {
	if (!argsSchema) return [];

	try {
		// Handle tuple-style arguments
		if (argsSchema._def?.typeName === 'ZodTuple') {
			const items = argsSchema._def.items || [];
			return items.map((item: ZodSchema, index: number) => {
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
	const [commands, setCommands] = useState([]);
	const [error, setError] = useState('');
	const [commandInfo, setCommandInfo] = useState(null);

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
								options: extractOptionInfo(commandModule.options as ZodSchema),
								args: extractArgumentInfo(
									commandModule.args as ZodSchema,
									commandModule.argsLabels || [],
								),
								usage: `steel ${command}${commandModule.args ? ' [arguments]' : ''}${commandModule.options ? ' [options]' : ''}`,
								template: commandModule.template, // Include template info if available
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
	}, [command]);

	// Show specific command help
	if (command) {
		return (
			<Box flexDirection="column" justifyContent="flex-start">
				<Box flexDirection="column">
					<Box marginBottom={1} flexDirection="column" gap={1}>
						<Text bold color="blue">
							Steel CLI - Help Menu
						</Text>
						<Box>
							<Text backgroundColor={'white'} color={'black'} bold>
								{' '}
								steel {command}{' '}
							</Text>
						</Box>
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

							{(command === 'run' || command === 'forge') &&
								TEMPLATES.length > 0 && (
									<>
										<Box marginBottom={1} marginTop={1}>
											<Text bold>TEMPLATES</Text>
										</Box>
										<Box paddingLeft={2} marginBottom={1}>
											<TemplateList templates={TEMPLATES} />
										</Box>
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
		<Box flexDirection="column">
			<CLIWelcomeMessage />
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
						<Text color="yellow">
							No commands found. Try using &lsquo;steel&rsquo;.
						</Text>
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
