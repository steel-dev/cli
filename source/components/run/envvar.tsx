import {useEffect, useState} from 'react';
import path from 'path';
import fs from 'fs';
import {v4 as uuidv4} from 'uuid';
import {Box, Text} from 'ink';
import TextInput from 'ink-text-input';
import {Task} from 'ink-task-list';
import spinners from 'cli-spinners';

import {getApiKey} from '../../utils/session.js';
import {updateEnvVariable} from '../../utils/cookbook.js';
import {ENV_VAR_MAP} from '../../utils/constants.js';
import {useRunStep} from '../../context/runstepcontext.js';
import {useTask} from '../../hooks/usetask.js';
// import type {Template} from '../../utils/types.js';

export default function EnvVar({options}: {options: any}) {
	const {step, setStep, template, envVars, setEnvVars, directory} =
		useRunStep();
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const workingDir = directory || process.cwd();

	// Manage interactive input state
	const [currentIndex, setCurrentIndex] = useState(0);
	const [inputValue, setInputValue] = useState('');

	// Derive required env vars based on static + dynamic dependencies
	const isEnvVarRequired = (varName: string, env: Record<string, string>) => {
		switch (varName) {
			case 'STEEL_API_KEY':
				return !env['STEEL_API_URL'];
			case 'STEEL_API_URL':
				return !env['STEEL_API_KEY'];
			default:
				return template.env.find(e => e.value === varName)?.required ?? false;
		}
	};
	const requiredEnvVars = template
		? template?.env?.filter(
				e => isEnvVarRequired(e.value, envVars) && !envVars[e.value],
			)
		: [];

	const currentPromptVar = template ? requiredEnvVars[currentIndex] : null;

	// Setup: preload values from options and write .env
	useEffect(() => {
		if (step !== 'envvar') return;

		setLoading(true);

		try {
			const apiKey = getApiKey();
			if (apiKey) setTask(apiKey);

			const envExamplePath = path.join(workingDir, '.env.example');
			const envTargetPath = path.join(workingDir, '.env');

			if (fs.existsSync(envExamplePath)) {
				fs.copyFileSync(envExamplePath, envTargetPath);

				for (const [key, envVar] of Object.entries(ENV_VAR_MAP)) {
					if (key in options) {
						updateEnvVariable(workingDir, envVar, String(options[key]));

						// Special case: set CONNECT_URL if api-url exists
						if (key === 'api-url') {
							updateEnvVariable(
								workingDir,
								'CONNECT_URL',
								'ws:' + options[key].split(':')[1],
							);
						}

						if (key === 'api-key') {
							setTask(String(options[key]));
						}
					}
				}

				// If api-url not provided, create a session ID
				if (!('api-url' in options)) {
					updateEnvVariable(workingDir, 'STEEL_SESSION_ID', uuidv4());
				}

				fs.unlinkSync(envExamplePath);
			}

			setLoading(false);
		} catch (error) {
			console.error('Error updating environment variables:', error);
			setError('Error updating environment variables');
			setLoading(false);
		}
	}, [step]);

	// Handle submission of individual env var inputs
	const handleInputSubmit = (val: string) => {
		const varName = currentPromptVar.value;

		setEnvVars({...envVars, [varName]: val});

		updateEnvVariable(workingDir, varName, val);

		if (currentIndex < requiredEnvVars.length - 1) {
			setCurrentIndex(index => index + 1);
			setInputValue('');
		} else {
			setStep('dependencies');
		}
	};

	// Done? Skip input
	if (requiredEnvVars.length === 0 || step !== 'envvar') {
		return (
			<Task
				label="Setting up environment variables"
				state={state}
				spinner={spinners.dots}
				isExpanded={step === 'envvar' && !task}
			>
				<Text color="green">✅ All required env vars set</Text>
			</Task>
		);
	}

	return (
		<Box flexDirection="column">
			<Task
				label="Setting up environment variables"
				state={state}
				spinner={spinners.dots}
				isExpanded={true}
			>
				{currentPromptVar && (
					<>
						<Text>
							🔧 Enter value for:
							<Text color="cyan">{currentPromptVar.label}</Text> (
							{currentPromptVar.value})
						</Text>
						<TextInput
							value={inputValue}
							onChange={setInputValue}
							onSubmit={handleInputSubmit}
							placeholder="Your value here..."
						/>
					</>
				)}
			</Task>
		</Box>
	);
}
