#!/usr/bin/env node

import React, {useState} from 'react';
import {Box, Text} from 'ink';
import SelectInput from 'ink-select-input';
import Callout from '../components/callout.js';
import {getSettings, setSettings} from '../utils/session.js';

export const description = 'Display current settings';

interface SettingItemProps {
	label: string;
	value: string;
	isSelected?: boolean;
}

function SettingItem({label, isSelected = false}: SettingItemProps) {
	const selectedColor = isSelected ? 'magenta' : 'cyan';

	return (
		<Box>
			<Text bold color={selectedColor}>
				{label}
			</Text>
		</Box>
	);
}

export default function Settings() {
	const settings = getSettings();
	const [sent, setSent] = useState(false);
	const currentInstance = settings?.instance || 'local';

	const items = [
		{
			label: currentInstance === 'cloud' ? 'Cloud (current)' : 'Cloud',
			value: 'cloud',
			isCurrent: currentInstance === 'cloud',
		},
		{
			label: currentInstance === 'local' ? 'Local (current)' : 'Local',
			value: 'local',
			isCurrent: currentInstance === 'local',
		},
	];

	return sent ? (
		<Callout variant="success" title="Settings Updated">
			Your settings have been saved successfully!
		</Callout>
	) : (
		<Box flexDirection="column">
			<Box marginBottom={1}>
				<Text color="dim">Select Instance Type:</Text>
			</Box>
			<SelectInput
				items={items}
				itemComponent={SettingItem}
				onSelect={item => {
					setSettings({instance: item.value});
					setSent(true);
				}}
			/>
		</Box>
	);
}
