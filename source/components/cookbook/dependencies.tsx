import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import spinners from 'cli-spinners';

export default function Dependencies({
	step,
	setStep,
}: {
	step: string;
	setStep: React.Dispatch<React.SetStateAction<string>>;
}) {
	//@ts-ignore
	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();
	useEffect(() => {
		let timer: NodeJS.Timeout;
		if (step === 'dependencies') {
			setLoading(true);
			try {
				console.log('Loading: ' + loading);
				timer = setTimeout(() => {
					setLoading(false);
					setTask(true);
					setStep('done');
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
			label="Installing dependencies"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
