import React from 'react';
import {Box, Text} from 'ink';
import SelectInput from 'ink-select-input';
import type {Template} from '../utils/types.js';

interface TemplatePickerProps {
	templates: Template[];
	onSelect: (template: Template) => void;
}

interface TemplateItemProps {
	label: string;
	value: string;
	command: string;
	language?: string;
	isSelected?: boolean;
}

function TemplateItem({
	command,
	label,
	language = '',
	isSelected = false,
}: TemplateItemProps) {
	const selectedColor = isSelected ? 'magenta' : 'cyan';
	const textColor = isSelected ? 'white' : 'gray';

	// Truncate and pad strings to fixed widths
	const paddedCommand = command.slice(0, 20).padEnd(20, ' ');
	const paddedLabel = label.slice(0, 42).padEnd(42, ' ');
	const paddedLanguage = language.slice(0, 6).padStart(6, ' ');

	return (
		<Box>
			<Text bold color={selectedColor}>
				{paddedCommand}
			</Text>
			<Text color={textColor}>{paddedLabel}</Text>
			<Text color={textColor}>{paddedLanguage}</Text>
		</Box>
	);
}

export default function TemplatePicker({
	templates,
	onSelect,
}: TemplatePickerProps) {
	// Add header row
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
			<SelectInput
				items={templates}
				itemComponent={TemplateItem}
				onSelect={onSelect}
			/>
		</Box>
	);
}
