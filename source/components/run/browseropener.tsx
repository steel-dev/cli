import React, {useEffect} from 'react';
import open from 'open';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function BrowserOpener({
	options,
	sessionId,
}: {
	options: any;
	sessionId?: string;
}) {
	const {step, setStep} = useStep();
	const [state, , , , setTask, setLoading, setError] = useTask();

	useEffect(() => {
		if (step === 'browser') {
			setLoading(true);
			try {
				// Determine if browser should be opened
				if (options.view || options.open) {
					// Determine URL based on sessionId - use local default if no sessionId provided
					const url = sessionId
						? `https://app.steel.dev/sessions/${sessionId}`
						: 'http://localhost:5167';

					// Open the browser
					open(url);
					setTask(`Opened ${url}`);
				} else {
					setTask('Skipped browser opening');
				}
				setLoading(false);
				setStep('complete'); // Move to next step, whatever that might be
			} catch (error) {
				console.error('Error opening browser:', error);
				setError('Error opening browser');
				setLoading(false);
			}
		}
	}, [step, sessionId, options]);

	return <Task label="Opening browser" state={state} spinner={spinners.dots} />;
}
