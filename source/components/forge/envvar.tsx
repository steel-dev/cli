import React, {useEffect, useState} from 'react';
import path from 'path';
import fs from 'fs';
import {Box, Text} from 'ink';
import TextInput from 'ink-text-input';
import {Task} from 'ink-task-list';
import spinners from 'cli-spinners';
import {getApiKey} from '../../utils/session.js';
import {updateEnvVariable} from '../../utils/forge.js';
import {ENV_VAR_MAP} from '../../utils/constants.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import {useTask} from '../../hooks/usetask.js';
import type {Options} from '../../commands/forge.js';

export default function EnvVar({options}: {options: Options}) {
	const {step, setStep, template, envVars, setEnvVars, directory} =
		useForgeStep();
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const workingDir = directory;
	// Manage interactive input state
	const [inputValue, setInputValue] = useState('');
	const [isCollectingVars, setIsCollectingVars] = useState(false);
	// Queue state for pending actions
	const [pendingVars, setPendingVars] = useState([]);
	// Derive required env vars
	const isEnvVarRequired = (varName: string, env: Record<string, string>) => {
		switch (varName) {
			case 'STEEL_API_KEY':
				return !env['STEEL_API_URL'];
			case 'STEEL_API_URL':
				if (inputValue !== '') {
					env['STEEL_CONNECT_URL'] = 'ws:' + inputValue.split(':')[1];
					updateEnvVariable(
						workingDir,
						'STEEL_CONNECT_URL',
						'ws:' + inputValue.split(':')[1],
					);
				}
				return !env['STEEL_API_KEY'];
			default:
				return template.env.find(e => e.value === varName)?.required ?? false;
		}
	};
	// Setup: preload values from options and write .env
	useEffect(() => {
		if (step === 'envvar' && !task && !isCollectingVars) {
			if (options.skip_auth) {
				setTask('skip_auth');
				setStep('dependencies');
				return;
			}
			setLoading(true);
			try {
				const envExamplePath = path.join(workingDir, '.env.example');
				const envTargetPath = path.join(workingDir, '.env');
				const curEnvVars = {};
				if (fs.existsSync(envExamplePath)) {
					fs.copyFileSync(envExamplePath, envTargetPath);
					for (const [key, envVar] of Object.entries(ENV_VAR_MAP)) {
						// console.log(key, envVar);
						if (key in options) {
							curEnvVars[envVar] = String(options[key]);
							updateEnvVariable(workingDir, envVar, String(options[key]));
							// Special case: set CONNECT_URL if api-url exists
							if (key === 'api_url') {
								curEnvVars['STEEL_CONNECT_URL'] =
									'ws:' + options[key].split(':')[1];
								updateEnvVariable(
									workingDir,
									'STEEL_CONNECT_URL',
									'ws:' + options[key].split(':')[1],
								);
							}
						}
					}
					const apiKey = getApiKey();
					if (apiKey) {
						curEnvVars['STEEL_API_KEY'] = apiKey.apiKey;
						updateEnvVariable(workingDir, 'STEEL_API_KEY', apiKey.apiKey);
					}
					setEnvVars(curEnvVars);
					const remaining = pendingVars.slice(1);
					setPendingVars(remaining);
					fs.unlinkSync(envExamplePath);
				}
				// Calculate which vars we still need after setup
				const stillNeeded =
					template?.env?.filter(
						e => isEnvVarRequired(e.value, curEnvVars) && !curEnvVars[e.value],
					) || [];
				// console.log(stillNeeded);
				if (stillNeeded.length > 0) {
					setPendingVars(stillNeeded);
					setIsCollectingVars(true);
				} else {
					setTask(curEnvVars);
					setStep('dependencies');
				}
				setLoading(false);
			} catch (error) {
				console.error('Error updating environment variables:', error);
				setError('Error updating environment variables');
				setLoading(false);
			}
		}
	}, [step, task, isCollectingVars]);
	// Handle submission of individual env var inputs
	const handleInputSubmit = (val: string) => {
		if (pendingVars.length === 0) return;
		const currentVar = pendingVars[0];
		const updatedEnvVars = {
			...envVars,
			[currentVar.value]:
				currentVar.value === 'STEEL_API_URL' && val === ''
					? 'https://api.steel.dev'
					: val,
		};
		setEnvVars(updatedEnvVars);
		updateEnvVariable(workingDir, currentVar.value, val);
		const remaining = pendingVars.slice(1);
		setPendingVars(remaining);
		setInputValue('');
		if (remaining.length === 0) {
			setIsCollectingVars(false);
			setTask(updatedEnvVars);
			setStep('dependencies');
		}
	};
	// Dequeue next action to be process and rendered
	const currentPromptVar = pendingVars[0] || null;

	return (
		<Box flexDirection="column">
			<Task
				label="Setting up environment variables"
				state={state}
				spinner={spinners.dots}
				isExpanded={step === 'envvar' && isCollectingVars}
			>
				{currentPromptVar && (
					<>
						<Text>
							🔧 Enter value for:
							<Text color="cyan"> {currentPromptVar.label}</Text> (
							{currentPromptVar.value})
						</Text>
						<TextInput
							value={inputValue}
							onChange={setInputValue}
							onSubmit={handleInputSubmit}
							placeholder={
								currentPromptVar.value === 'STEEL_API_URL'
									? 'https://api.steel.dev'
									: 'Your value here...'
							}
						/>
					</>
				)}
			</Task>
		</Box>
	);
}
