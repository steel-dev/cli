import React, {useEffect, useState} from 'react';
import path from 'path';
import fs from 'fs';
import {v4 as uuidv4} from 'uuid';
import {Box, Text} from 'ink';
import TextInput from 'ink-text-input';
import {Task} from 'ink-task-list';
import spinners from 'cli-spinners';
import {getApiKey} from '../../utils/session.js';
import {ENV_VAR_MAP} from '../../utils/constants.js';
import {useRunStep} from '../../context/runstepcontext.js';
import {useTask} from '../../hooks/usetask.js';
import type {Options} from '../../commands/run.js';

export default function EnvVar({options}: {options: Options}) {
	const {
		step,
		setStep,
		template,
		envVars,
		setEnvVars,
		directory,
		setSessionId,
	} = useRunStep();
	const [state, task, , , setTask, setLoading, setError] = useTask();
	// Manage interactive input state
	const [inputValue, setInputValue] = useState('');
	const [isCollectingVars, setIsCollectingVars] = useState(false);
	// Queue state for pending actions
	const [pendingVars, setPendingVars] = useState([]);
	// Derive required env vars
	const isEnvVarRequired = (varName, env) => {
		switch (varName) {
			case 'STEEL_API_KEY':
				return !env['STEEL_API_URL'];
			case 'STEEL_API_URL':
				if (inputValue !== '') {
					env['STEEL_CONNECT_URL'] = 'ws:' + inputValue.split(':')[1];
				}
				return !env['STEEL_API_KEY'];
			default:
				return template.env.find(e => e.value === varName)?.required ?? false;
		}
	};
	// Setup: preload values from options and write .env
	useEffect(() => {
		if (step === 'envvar' && !task && !isCollectingVars) {
			setLoading(true);
			try {
				const curEnvVars = {};
				const apiKey = getApiKey();
				if (apiKey) {
					curEnvVars['STEEL_API_KEY'] = apiKey.apiKey;
				}
				console.log('cur:', curEnvVars);
				const envExamplePath = path.join(directory, '.env.example');
				if (fs.existsSync(envExamplePath)) {
					for (const [key, envVar] of Object.entries(ENV_VAR_MAP)) {
						console.log('Key:', key, 'Value:', envVar);
						if (key in options) {
							curEnvVars[envVar] = String(options[key]);
							// Special case: set CONNECT_URL if api-url exists
							if (key === 'api_url') {
								curEnvVars['STEEL_CONNECT_URL'] =
									'ws:' + options[key].split(':')[1];
							}
							console.log('cur:', curEnvVars);
						}
					}
					const remaining = pendingVars.slice(1);
					setPendingVars(remaining);
					// If api-url not provided, create a session ID
					if (!('api_url' in options)) {
						const sessionId = uuidv4();
						setSessionId(sessionId);
						curEnvVars['STEEL_SESSION_ID'] = sessionId;
					}
					fs.unlinkSync(envExamplePath);
				}
				console.log(curEnvVars);
				// Calculate which vars we still need after setup
				const stillNeeded =
					template?.env?.filter(
						e => isEnvVarRequired(e.value, curEnvVars) && !curEnvVars[e.value],
					) || [];
				setEnvVars(curEnvVars);
				if (stillNeeded.length > 0) {
					setPendingVars(stillNeeded);
					setIsCollectingVars(true);
				} else {
					setTask(envVars);
					setIsCollectingVars(false);
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
	const handleInputSubmit = val => {
		if (pendingVars.length === 0) return;
		const currentVar = pendingVars[0];
		const updatedEnvVars = {...envVars, [currentVar.value]: val};
		setEnvVars(updatedEnvVars);
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
				isExpanded={step === 'envvar' && !task && isCollectingVars}
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
							placeholder="Your value here..."
						/>
					</>
				)}
			</Task>
		</Box>
	);
}
