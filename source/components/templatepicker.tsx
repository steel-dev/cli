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
	alias: string;
	language?: string;
	isSelected?: boolean;
}

function TemplateItem({
	alias,
	label,
	language = '',
	isSelected = false,
}: TemplateItemProps) {
	const selectedColor = isSelected ? 'magenta' : 'cyan';
	const textColor = isSelected ? 'white' : 'gray';

	// Truncate and pad strings to fixed widths
	const paddedAlias = alias.slice(0, 16).padEnd(16, ' ');
	const paddedLabel = label.slice(0, 45).padEnd(45, ' ');
	const paddedLanguage = language.slice(0, 6).padStart(6, ' ');

	return (
		<Box>
			<Text bold color={selectedColor}>
				{paddedAlias}
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
			<Text color="dim">{'Command'.padEnd(16, ' ')}</Text>
			<Text color="dim">{'Label'.padEnd(45, ' ')}</Text>
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
