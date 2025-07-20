import path from 'path';
import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {spawn} from 'child_process';
import {CACHE_DIR} from '../../utils/constants.js';
import type {Options} from '../../commands/run.js';

export default function Runner({options}: {options: Options}) {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, template, directory, hash} = useRunStep();
	useEffect(() => {
		if (step === 'runner' && !task) {
			setLoading(true);
			if (!template?.runCommand) {
				setError('No run command specified in template');
				setLoading(false);
			}
			try {
				// Parse the command into the binary and its arguments
				const command = template.runCommand({
					depsDir: path.join(CACHE_DIR, hash),
				});
				const child = spawn(command, {
					cwd: directory,
					shell: true,
					stdio: 'inherit', // This will pipe stdout and stderr to the parent process
					env: {...envVars, ...process.env},
				});
				process.on('SIGINT', () => {
					child.kill();
				});
				setTask(`Command completed successfully: ${template.runCommand}`);
				if (options.view) {
					setStep('browser');
				} else {
					setStep('done');
				}
				setLoading(false);
			} catch (error) {
				console.error('Error running command:', error);
				setError(`Error running command: ${error.message}`);
				setLoading(false);
			}
		}
	}, [step, template]);

	return (
		<Task
			label={`Running ${template?.label ? `${template.label} ` : ''}example`}
			state={state}
			spinner={spinners.dots}
		/>
	);
}
