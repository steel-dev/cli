import React from 'react';
import {Box, Text} from 'ink';
import type {Template} from '../utils/types.js';

interface TemplateListProps {
	templates: Template[];
}

interface TemplateItemProps {
	command: string;
	label: string;
	language?: string;
}

function TemplateItem({command, label, language = ''}: TemplateItemProps) {
	// Truncate and pad strings to fixed widths (same as TemplatePicker)
	const paddedCommand = command.slice(0, 20).padEnd(20, ' ');
	const paddedLabel = label.slice(0, 42).padEnd(42, ' ');
	const paddedLanguage = language.slice(0, 6).padStart(6, ' ');

	return (
		<Box>
			<Text color="cyan">{paddedCommand}</Text>
			<Text color="gray">{paddedLabel}</Text>
			<Text color="gray">{paddedLanguage}</Text>
		</Box>
	);
}

export default function TemplateList({templates}: TemplateListProps) {
	// Add header row (same as TemplatePicker)
	const HeaderRow = () => (
		<Box marginBottom={1}>
			<Text>{'  '}</Text>
			<Text color="dim">{'Command'.padEnd(20, ' ')}</Text>
			<Text color="dim">{'Label'.padEnd(42, ' ')}</Text>
			<Text color="dim">{'Lang.'.padStart(6, ' ')}</Text>
		</Box>
	);

	return (
		<Box flexDirection="column">
			<HeaderRow />
			{templates.map(template => (
				<Box key={template.alias} paddingLeft={2}>
					<TemplateItem
						command={template.command}
						label={template.label}
						language={template.language}
					/>
				</Box>
			))}
		</Box>
	);
}
