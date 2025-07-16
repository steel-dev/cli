#!/usr/bin/env node

import React, {ReactElement, useState} from 'react';
import {Form} from 'ink-form';
import {Box, Text} from 'ink';
import {getSettings, setSettings} from '../utils/session.js';

export const description = 'Display current settings';

export default function Settings(): ReactElement {
	const settings = getSettings();
	const [sent, setSent] = useState(false);
	return sent ? (
		<Box>
			<Text color="green">Settings saved!</Text>
		</Box>
	) : (
		<Form
			form={{
				title: 'Settings',
				sections: [
					{
						title: 'API Usage',
						fields: [
							{
								name: 'instance',
								label: 'Instance Type',
								type: 'select',
								required: true,
								initialValue: settings?.instance || 'cloud',
								options: [
									{label: 'Cloud', value: 'cloud'},
									{label: 'Local', value: 'local'},
								],
							},
						],
					},
				],
			}}
			onSubmit={value => {
				setSettings(value);
				setSent(true);
			}}
		/>
	);
}
