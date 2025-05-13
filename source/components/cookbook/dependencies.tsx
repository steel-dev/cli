import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import {spawn} from 'node:child_process';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();

	const {
		step,
		setStep,
		packageManager,
		// directory,
		// setDirectory,
		// template,
		// setTemplate,
	} = useStep();

	useEffect(() => {
		let timer: NodeJS.Timeout;
		if (step === 'dependencies' && !task) {
			setLoading(true);
			try {
				spawn(packageManager, ['install']);
				timer = setTimeout(() => {
					setLoading(false);
					setTask(true);
					setStep('done');
				}, 2000);
			} catch (error) {
				console.error('Error installing dependencies:', error);
				setError('Error installing dependencies');
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
