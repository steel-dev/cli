import path from 'path';
import React, {useEffect} from 'react';
import {getApiKey} from '../../utils/session.js';
import {updateEnvVariable} from '../../utils/cookbook.js';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import fs from 'fs';

export default function SteelApiKey() {
	const {step, setStep, directory} = useStep();

	const [state, , , , setTask, setLoading, setError] = useTask();

	useEffect(() => {
		if (step === 'apikey') {
			setLoading(true);
			try {
				const apiKey = getApiKey();
				if (apiKey) {
					setTask(apiKey);
				} else {
					setTask('your-api-key-here');
				}
				// I want to write the env variable to .env file
				// copy all variables over from .env.example to .env file and replace STEEL_API_KEY with the actual API key
				const envPath = path.join(directory, '.env.example');
				if (fs.existsSync(envPath)) {
					fs.copyFileSync(envPath, path.join(directory, '.env'));
					updateEnvVariable(
						directory,
						'STEEL_API_KEY',
						apiKey?.apiKey || 'your-api-key-here',
					);
					fs.unlinkSync(envPath);
				}
				setLoading(false);
				setStep('dependencies');
			} catch (error) {
				console.error('Error fetching API key:', error);
				setError('Error fetching API key');
				setLoading(false);
			}
		}
	}, [step]);

	return (
		<Task
			label="Grabbing Steel API Key"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
