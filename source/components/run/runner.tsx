import path from 'path';
import React, {useEffect, useState} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {spawn} from 'child_process';
import {CACHE_DIR} from '../../utils/constants.js';
import type {Options} from '../../commands/run.js';
import open from 'open';

export default function Runner({options}: {options: Options}) {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, template, directory, hash, sessionId} =
		useRunStep();
	const [output, setOutput] = useState<string>('');
	const [isOpen, setIsOpen] = useState(false);

	useEffect(() => {
		if (step === 'runner' && !task) {
			setLoading(true);
			if (!template?.runCommand) {
				setError('No run command specified in template');
				setOutput('No run command specified in template');
				setLoading(false);
				return;
			}
			try {
				// Parse the command into the binary and its arguments
				const command = template.runCommand({
					depsDir: path.join(CACHE_DIR, hash),
				});
				const child = spawn(command, {
					cwd: directory,
					shell: true,
					stdio: ['pipe', 'pipe', 'pipe'], // Capture stdout and stderr
					env: {...envVars, ...process.env},
				});

				// Stream stdout to output state
				child.stdout?.on('data', data => {
					const text = data.toString();
					setOutput(prev => prev + text);
					if (
						((output + text).includes('https://app.steel.dev') ||
							(output + text).includes('http://localhost:5173')) &&
						options.view &&
						!isOpen
					) {
						try {
							// Determine URL based on sessionId - use local default if no sessionId provided
							const url = sessionId
								? `https://app.steel.dev/sessions/${sessionId}`
								: 'http://localhost:5173';

							open(url);
							setIsOpen(true);
						} catch {
							setError('Error opening browser');
							setLoading(false);
						}
					}
				});

				// Stream stderr to output state
				child.stderr?.on('data', data => {
					const text = data.toString();
					setOutput(prev => prev + text);
				});

				child.on('close', code => {
					if (code === 0) {
						setTask(`Command completed successfully: ${template.runCommand}`);
						setStep('done');
					} else {
						console.log(output);
						setError(`Command failed with exit code ${code}`);
						setOutput(`Command failed with exit code ${code}`);
					}
					setLoading(false);
				});

				child.on('error', error => {
					console.error('Error running command:', error);
					setError(`Error running command: ${error.message}`);
					setOutput(`Error running command: ${error.message}`);
					setLoading(false);
				});

				process.on('SIGINT', () => {
					child.kill();
				});
			} catch (error) {
				console.error('Error running command:', error);
				setError(`Error running command: ${error.message}`);
				setOutput(`Error running command: ${error.message}`);
				setLoading(false);
			}
		}
	}, [step, template]);

	return (
		<Task
			label={`Running ${template?.label ? `${template.label} ` : ''}example`}
			state={state}
			spinner={spinners.dots}
			output={output}
		/>
	);
}
