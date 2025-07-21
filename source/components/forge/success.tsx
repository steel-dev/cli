import React from 'react';
import {useForgeStep} from '../../context/forgestepcontext.js';
import Callout from '../callout.js';
import {Text} from 'ink';

export default function ForgeSuccess() {
	const {step, template, directory, packageManager} = useForgeStep();

	if (step !== 'success') return null;

	const runCommands =
		template?.displayRunCommands?.({
			directory,
			packageManager,
		}) || [];

	return (
		<Callout variant="success" title="Run your project">
			{runCommands.map((command, index) => (
				<Text key={index}>
					{index > 0 && <br />} {command}
					{'\n'}
				</Text>
			))}
		</Callout>
	);
}
