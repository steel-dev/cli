import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function ProjectScaffolding() {
	//@ts-ignore
	const {step, setStep, directory, setDirectory, template, setTemplate} =
		useStep();
	//@ts-ignore
	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();

	useEffect(() => {
		let timer: NodeJS.Timeout;
		if (step === 'scaffold') {
			setLoading(true);
			try {
				timer = setTimeout(() => {
					setLoading(false);
					setTask(true);
					setStep('apikey');
				}, 3500);
			} catch (error) {
				console.error('Error fetching API key:', error);
				setError('Error fetching API key');
				setLoading(false);
			}
		}
		return () => clearTimeout(timer);
	}, [step]);

	return (
		<Task label="Scaffolding project" state={state} spinner={spinners.dots} />
	);
}
