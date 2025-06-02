import React, {ReactElement} from 'react';
import {Form} from 'ink-form';
import {getSettings, setSettings} from '../utils/session.js';

export const description = 'Display current settings';

export default function Settings(): ReactElement {
	const settings = getSettings();
	return (
		<Form
			form={{
				title: 'Settings',
				sections: [
					{
						title: 'API Usage',
						fields: [
							{
								name: 'Instance Type',
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
			onSubmit={value => setSettings(value)}
		/>
	);
}
