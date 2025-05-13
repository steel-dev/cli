//@ts-nocheck
import React, {useEffect} from 'react';
import {getApiKey} from '../../utils/session.js';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function SteelApiKey() {
	const {step, setStep, directory, setDirectory, template, setTemplate} =
		useStep();

	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();

	useEffect(() => {
		let timer: NodeJS.Timeout;
		if (step === 'apikey') {
			setLoading(true);
			try {
				timer = setTimeout(() => {
					const apiKey = getApiKey();
					if (apiKey) {
						setTask(apiKey);
					} else {
					}
					setLoading(false);
					setStep('dependencies');
				}, 2000);
			} catch (error) {
				console.error('Error fetching API key:', error);
				setError('Error fetching API key');
				setLoading(false);
			}
		}
		return () => clearTimeout(timer);
	}, [step]);

	return (
		<Task
			label="Grabbing Steel API Key"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
