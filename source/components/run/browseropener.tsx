import React, {useEffect} from 'react';
import open from 'open';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import type {Options} from '../../commands/run.js';

export default function BrowserOpener({options}: {options: Options}) {
	const {step, sessionId} = useRunStep();
	const [state, task, , , setTask, setLoading, setError] = useTask();
	useEffect(() => {
		if (step === 'browser' && !task) {
			setLoading(true);
			try {
				// Determine URL based on sessionId - use local default if no sessionId provided
				const url = sessionId
					? `https://app.steel.dev/sessions/${sessionId}`
					: 'http://localhost:5173';
				if (options.view) {
					// Open the browser
					open(url);
					setTask(`Opened ${url}`);
				} else {
					setTask('skipped');
				}
				setLoading(false);
				// setStep('done');
			} catch (error) {
				console.error('Error opening browser:', error);
				setError('Error opening browser');
				setLoading(false);
			}
		}
	}, [step, sessionId, options]);

	return (
		<Task
			label="Opening browser"
			state={state}
			spinner={spinners.dots}
			isExpanded={task === 'browser'}
		/>
	);
}
