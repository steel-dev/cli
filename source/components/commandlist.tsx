import React from 'react';
import {Box, Text} from 'ink';

type Props = {
	commands: Array<{
		name: string;
		description: string;
		isDefault?: boolean;
		options?: unknown;
		args?: unknown;
	}>;
};

export default function CommandList({commands}: Props) {
	// Define command categories
	const quickstartCommands = ['forge', 'run'];
	const apiEndpoints = ['sessions', 'files', 'tools'];

	// First, let's build a proper hierarchy
	const buildHierarchy = commandList => {
		const tree = {};

		commandList.forEach(cmd => {
			const parts = cmd.name.split(' ');
			const isIndex = cmd.name.endsWith(' index');

			let current = tree;
			let path = '';

			// Build the path through the tree
			for (let i = 0; i < parts.length; i++) {
				const part = parts[i];
				path = path ? `${path} ${part}` : part;

				if (part === 'index') {
					// This is an index file, mark the parent as a folder
					continue;
				}

				if (!current[part]) {
					current[part] = {
						name: part,
						fullName: path,
						children: {},
						command: null,
						isFolder: false,
					};
				}

				// If this is the last part and not an index, it's a command
				if (i === parts.length - 1 && !isIndex) {
					current[part].command = cmd;
				}

				// Check if there's an index file for this path
				const indexCmd = commandList.find(c => c.name === `${path} index`);
				if (indexCmd && parts.length === 1) {
					current[part].isFolder = true;
					current[part].description = indexCmd.description;
				}

				current = current[part].children;
			}
		});

		return tree;
	};

	// Flatten the hierarchy in the correct order
	const flattenHierarchy = (node, depth = 0, result = []) => {
		Object.keys(node)
			.sort()
			.forEach(key => {
				const item = node[key];

				// Add the folder/command to results
				result.push({
					name: item.fullName,
					displayName: item.name,
					description: item.description || item.command?.description || '',
					depth: depth,
					isFolder: item.isFolder,
					command: item.command,
				});

				// Recursively add children
				if (Object.keys(item.children).length > 0) {
					flattenHierarchy(item.children, depth + 1, result);
				}
			});

		return result;
	};

	// Separate commands into three categories
	const quickstartCmds = commands.filter(cmd => {
		const rootCommand = cmd.name.split(' ')[0];
		return quickstartCommands.includes(rootCommand);
	});

	const apiCommands = commands.filter(cmd => {
		const rootCommand = cmd.name.split(' ')[0];
		return apiEndpoints.includes(rootCommand);
	});

	const otherCommands = commands.filter(cmd => {
		const rootCommand = cmd.name.split(' ')[0];
		return (
			!quickstartCommands.includes(rootCommand) &&
			!apiEndpoints.includes(rootCommand)
		);
	});

	// Build hierarchies for all three groups
	const quickstartHierarchy = buildHierarchy(quickstartCmds);
	const apiHierarchy = buildHierarchy(apiCommands);
	const otherHierarchy = buildHierarchy(otherCommands);

	const processedQuickstartCommands = flattenHierarchy(quickstartHierarchy);
	const processedApiCommands = flattenHierarchy(apiHierarchy);
	const processedOtherCommands = flattenHierarchy(otherHierarchy);

	const renderCommands = commandList => {
		return commandList.map(command => {
			const paddingLeft = 2 + command.depth * 3;
			const isSubCommand = command.depth > 0;

			return (
				<Box key={command.name} paddingLeft={paddingLeft} marginBottom={0}>
					<Box width={Math.max(20 - command.depth * 2, 8)}>
						<Text color={command.isFolder || isSubCommand ? 'blue' : 'cyan'}>
							{command.isFolder ? (
								<Text>
									<Text color="cyan">└─ </Text>
									{command.displayName}
								</Text>
							) : isSubCommand ? (
								<Text>
									<Text color="cyan">└─ </Text>
									{command.displayName}
								</Text>
							) : (
								command.displayName
							)}
						</Text>
					</Box>
					<Text>{command.description}</Text>
				</Box>
			);
		});
	};

	return (
		<>
			{processedQuickstartCommands.length > 0 && (
				<>
					<Box marginBottom={1} marginTop={1}>
						<Text bold color="yellow">
							⚡ Quickstart Commands
						</Text>
					</Box>
					{renderCommands(processedQuickstartCommands)}
				</>
			)}

			{processedApiCommands.length > 0 && (
				<>
					<Box
						marginBottom={1}
						marginTop={processedQuickstartCommands.length > 0 ? 2 : 1}
					>
						<Text bold color="yellow">
							🌐 API Endpoints
						</Text>
					</Box>
					{renderCommands(processedApiCommands)}
				</>
			)}

			{processedOtherCommands.length > 0 && (
				<>
					<Box
						marginBottom={1}
						marginTop={
							processedQuickstartCommands.length > 0 ||
							processedApiCommands.length > 0
								? 2
								: 1
						}
					>
						<Text bold color="yellow">
						⏺︎ Other Commands
						</Text>
					</Box>
					{renderCommands(processedOtherCommands)}
				</>
			)}
		</>
	);
}
