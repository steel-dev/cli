import path from 'path';
import React, {useEffect, useState, useRef} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {spawn, ChildProcess} from 'child_process';
import {CACHE_DIR} from '../../utils/constants.js';
import type {Options} from '../../commands/run.js';
import open from 'open';

export default function Runner({options}: {options: Options}) {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, template, directory, hash, sessionId} =
		useRunStep();
	const [output, setOutput] = useState<string>('');
	const [isOpen, setIsOpen] = useState(false);
	const childProcessRef = useRef<ChildProcess | null>(null);

	const cleanupChildProcess = () => {
		if (childProcessRef.current && !childProcessRef.current.killed) {
			childProcessRef.current.kill('SIGTERM');

			setTimeout(() => {
				if (childProcessRef.current && !childProcessRef.current.killed) {
					childProcessRef.current.kill('SIGKILL');
				}
				childProcessRef.current = null;
			}, 2000);
		}
	};

	useEffect(() => {
		return () => {
			cleanupChildProcess();
		};
	}, []);

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
				const command = template.runCommand({
					depsDir: path.join(CACHE_DIR, hash),
				});

				const child = spawn(command, {
					cwd: directory,
					shell: true,
					stdio: ['pipe', 'pipe', 'pipe'],
					env: {
						...process.env,
						...envVars,
						PYTHONUNBUFFERED: '1',
					},
					detached: false,
				});

				childProcessRef.current = child;

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

				child.on('close', (code, signal) => {
					childProcessRef.current = null;

					if (code === 0) {
						setTask(`Command completed successfully`);
						setStep('done');
					} else if (signal) {
						setError(`Command terminated by signal: ${signal}`);
						setOutput(`Command terminated by signal: ${signal}`);
					} else if (code === null) {
						setError('Command was terminated unexpectedly');
						setOutput('Command was terminated unexpectedly');
					} else {
						setError(
							`Command failed with exit code ${code} try clearing cache with 'steel cache --clean' and running again`,
						);
						setOutput(
							`Command failed with exit code ${code}, try clearing cache with 'steel cache --clean' and running again`,
						);
					}
					setLoading(false);
				});

				child.on('error', error => {
					childProcessRef.current = null;

					console.error('Error running command:', error);
					setError(
						`Error running command: ${error.message}, try clearing cache with 'steel cache --clean' and running again`,
					);
					setOutput(
						`Error running command: ${error.message}, try clearing cache with 'steel cache --clean' and running again`,
					);
					setLoading(false);
				});

				return () => {
					cleanupChildProcess();
				};
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
