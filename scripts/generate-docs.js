#!/usr/bin/env node

import fs from 'fs/promises';
import path from 'path';
import {fileURLToPath} from 'url';
import {dirname} from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = path.resolve(__dirname, '../');

// Import templates from constants
async function getTemplates() {
	try {
		const constantsPath = path.join(projectRoot, 'dist/utils/constants.js');
		const constants = await import(`file://${constantsPath}`);
		return constants.TEMPLATES || [];
	} catch (error) {
		console.error('Failed to load templates:', error);
		return [];
	}
}

// Function to scan command directory and get all available commands
async function getAvailableCommands(baseDir = 'commands') {
	try {
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
			if (
				file.name.startsWith('_') ||
				file.name === 'help.js' ||
				file.name === 'index.js'
			)
				continue;

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
					argsLabels: commandModule.argsLabels,
				});
			}
		}
		return commands;
	} catch (error) {
		console.error('Error scanning commands:', error);
		return [];
	}
}

// Function to get command module for a specific command
async function getCommandModule(commandPath) {
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
function extractOptionInfo(optionSchema) {
	if (!optionSchema) return [];

	try {
		const toCliOptionName = name => {
			if (name.includes('_')) {
				return name;
			}

			return name.replaceAll(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
		};

		const shapeFn = optionSchema._def?.shape;
		if (!shapeFn) return [];

		// If shape is a function, call it to get the actual shape
		const shape = typeof shapeFn === 'function' ? shapeFn() : shapeFn;
		if (!shape) return [];

		return Object.entries(shape).map(([name, def]) => {
			const description = def._def?.description || '';
			const type = def._def?.typeName || '';

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
				} catch {
					// Ignore parsing errors
				}
			}

			// Get readable description text
			let desc = description;

			// Handle __pastel_option_config__ format
			if (desc.includes('__pastel_option_config__')) {
				try {
					const configMatch = desc.match(/__pastel_option_config__({.*})/);
					if (configMatch) {
						const config = JSON.parse(configMatch[1]);
						desc = config.description || '';
						if (!alias && config.alias) {
							alias = config.alias;
						}
					}
				} catch {
					// If parsing fails, try to extract description from the raw string
					const descMatch = desc.match(/"description"\s*:\s*"([^"]+)"/);
					if (descMatch) {
						desc = descMatch[1];
					}
				}
			} else if (desc.startsWith('{')) {
				try {
					const parsed = JSON.parse(desc);
					desc = parsed.description || '';
				} catch {
					// If parsing fails, use the original description
				}
			}

			// Check if option is optional
			const isOptional = def._def?.typeName === 'ZodOptional';

			return {
				name: toCliOptionName(name),
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
function extractArgumentInfo(argsSchema, argsLabels) {
	if (!argsSchema) return [];

	try {
		// Handle tuple-style arguments
		if (argsSchema._def?.typeName === 'ZodTuple') {
			const items = argsSchema._def.items || [];
			return items.map((item, index) => {
				const description = item._def?.description || '';
				const isOptional = item._def?.typeName === 'ZodOptional';

				// Parse name and description from the description string
				let name =
					argsLabels && argsLabels[index]
						? argsLabels[index]
						: `args${index + 1}`;
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

// Generate markdown for a single command
async function generateCommandMarkdown(commandName, commands, templates) {
	try {
		const commandModule = await getCommandModule(commandName);

		if (!commandModule) {
			return '';
		}

		const options = extractOptionInfo(commandModule.options);
		const args = extractArgumentInfo(
			commandModule.args,
			commandModule.argsLabels,
		);
		const description = commandModule.description || '';

		let markdown = `## steel ${commandName}\n\n`;

		if (description) {
			markdown += `${description}\n\n`;
		}

		// Usage section
		markdown += `### Usage\n\n\`\`\`\nsteel ${commandName}`;
		if (args && args.length > 0) {
			args.forEach(arg => {
				if (arg.isOptional) {
					markdown += ` [${arg.name}]`;
				} else {
					markdown += ` <${arg.name}>`;
				}
			});
		}
		if (options && options.length > 0) {
			markdown += ' [options]';
		}
		markdown += '\n```\n\n';

		// Available Templates section for run and forge commands
		if (
			(commandName === 'run' || commandName === 'forge') &&
			templates.length > 0
		) {
			markdown += `### Available Templates\n\n`;
			markdown += `| Alias | Label | Language | Description |\n`;
			markdown += `|-------|-------|----------|-------------|\n`;

			templates.forEach(template => {
				markdown += `| \`${template.alias}\` | ${template.label} | ${template.language} | Template for ${template.label} automation |\n`;
			});
			markdown += '\n';
		}

		// Arguments section
		if (args && args.length > 0) {
			markdown += `### Arguments\n\n`;
			args.forEach(arg => {
				markdown += `- **${arg.name}**`;
				if (arg.isOptional) {
					markdown += ' (optional)';
				}
				if (arg.description) {
					markdown += `: ${arg.description}`;
				}
				markdown += '\n';
			});
			markdown += '\n';
		}

		// Options section
		if (options && options.length > 0) {
			markdown += `### Options\n\n`;
			options.forEach(opt => {
				markdown += `- `;
				if (opt.alias) {
					markdown += `-${opt.alias}, `;
				}
				markdown += `**--${opt.name}**`;
				if (opt.description) {
					markdown += `: ${opt.description}`;
				}
				markdown += '\n';
			});
			markdown += '\n';
		}

		return markdown;
	} catch (error) {
		console.error(`Error generating docs for command ${commandName}:`, error);
		return '';
	}
}

// Main function to generate complete CLI reference
async function generateCliReference() {
	try {
		console.log('🔍 Scanning commands...');
		const commands = await getAvailableCommands();

		console.log('📦 Loading templates...');
		const templates = await getTemplates();

		console.log('📝 Generating documentation...');

		let markdown = `# Steel CLI Reference\n\n`;
		markdown += `This is an auto-generated reference for the Steel CLI. The Steel CLI helps you create, run, and manage browser automation projects in the cloud.\n\n`;

		// Table of contents
		markdown += `## Table of Contents\n\n`;

		// Categorize commands
		const topLevelCommands = commands.filter(cmd => !cmd.name.includes(' '));
		const subCommands = commands.filter(cmd => cmd.name.includes(' '));

		// Top-level commands in TOC
		topLevelCommands.forEach(cmd => {
			markdown += `- [steel ${cmd.name}](#steel-${cmd.name.replace(/\s+/g, '-')})\n`;
		});

		// Sub-commands in TOC
		subCommands.forEach(cmd => {
			markdown += `- [steel ${cmd.name}](#steel-${cmd.name.replace(/\s+/g, '-')})\n`;
		});

		markdown += '\n';

		// Global Options
		markdown += `## Global Options\n\n`;
		markdown += `These options are available for most commands:\n\n`;
		markdown += `- **-h, --help**: Display help for a command\n`;
		markdown += `- **-v, --version**: Display Steel CLI version\n\n`;

		// Generate documentation for each command
		for (const command of [...topLevelCommands, ...subCommands]) {
			const commandMarkdown = await generateCommandMarkdown(
				command.name,
				commands,
				templates,
			);
			markdown += commandMarkdown;
		}

		// Footer
		markdown += `---\n\n`;
		markdown += `*This documentation was auto-generated from the Steel CLI source code.*\n`;

		// Write to file
		const docsPath = path.join(projectRoot, 'docs', 'cli-reference.md');
		await fs.writeFile(docsPath, markdown, 'utf8');

		console.log(`✅ CLI reference generated at: ${docsPath}`);
		console.log(
			`📊 Generated docs for ${commands.length} commands and ${templates.length} templates`,
		);
	} catch (error) {
		console.error('❌ Error generating CLI reference:', error);
		process.exit(1);
	}
}

// Run the generator
generateCliReference();
