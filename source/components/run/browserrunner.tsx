import React, {useEffect, useState, useRef} from 'react';
import {spawn, ChildProcess} from 'child_process';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import {CONFIG_DIR, REPO_URL} from '../../utils/constants.js';
import path from 'path';
import spinners from 'cli-spinners';

function isDockerRunning() {
	try {
		const child = spawn('docker', ['info'], {stdio: 'ignore'});
		return new Promise<boolean>(resolve => {
			child.on('exit', code => {
				resolve(code === 0);
			});
			child.on('error', () => {
				resolve(false);
			});
		});
	} catch {
		return Promise.resolve(false);
	}
}

export default function BrowserRunner() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars} = useRunStep();
	const [output, setOutput] = useState<string>('');
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
		if (step === 'browserrunner' && !task) {
			setLoading(true);

			async function startBrowserRunner() {
				try {
					if (!envVars['STEEL_API_URL'].includes('localhost')) {
						setTask('Skipping Docker Compose');
						setStep('runner');
						setLoading(false);
						return;
					}

					const cloneChild = spawn('git', ['clone', REPO_URL], {
						cwd: CONFIG_DIR,
						stdio: 'ignore',
					});

					await new Promise<void>((resolve, reject) => {
						cloneChild.on('exit', code => {
							if (code === 0 || code === 128) {
								// 128 = already exists
								resolve();
							} else {
								reject(new Error(`Git clone failed with code ${code}`));
							}
						});
						cloneChild.on('error', reject);
					});

					const dockerRunning = await isDockerRunning();
					if (!dockerRunning) {
						console.log(
							'⚠️ Docker is not running. Please start it and try again.',
						);
						setError('Docker is not running');
						setLoading(false);
						return;
					}

					console.log('🚀 Starting Docker Compose...');
					const folderName = path.basename(REPO_URL, '.git');
					const child = spawn(
						'docker-compose',
						['-f', 'docker-compose.dev.yml', 'up', '-d'],
						{
							cwd: path.join(CONFIG_DIR, folderName),
							stdio: ['pipe', 'pipe', 'pipe'],
							env: {
								...process.env,
								API_PORT: String(
									Number(envVars['STEEL_API_URL'].split(':')[2]) ||
										Number(envVars['STEEL_API_URL'].split(':')[1]) ||
										3000,
								),
							},
						},
					);

					childProcessRef.current = child;

					// Stream stdout to output state
					child.stdout?.on('data', data => {
						const text = data.toString();
						setOutput(prev => prev + text);
					});

					// Stream stderr to output state
					child.stderr?.on('data', data => {
						const text = data.toString();
						setOutput(prev => prev + text);
					});

					child.on('close', (code, signal) => {
						childProcessRef.current = null;

						if (code === 0) {
							setTask('Docker Compose started successfully');
							setStep('runner');
						} else if (signal) {
							setError(`Docker Compose terminated by signal: ${signal}`);
						} else {
							setError(`Docker Compose failed with exit code ${code}`);
						}
						setLoading(false);
					});

					child.on('error', error => {
						childProcessRef.current = null;
						setError(`Error starting Steel Browser Locally: ${error.message}`);
						setLoading(false);
					});
				} catch (error) {
					console.error('Error in browser runner:', error);
					setError(`Error starting Steel Browser Locally: ${error.message}`);
					setLoading(false);
				}
			}

			startBrowserRunner();
		}
	}, [step]);

	return (
		<Task
			label={`Running Steel Browser locally`}
			state={state}
			spinner={spinners.dots}
			isExpanded={state === 'loading'}
			output={state === 'loading' ? output : undefined}
		/>
	);
}
