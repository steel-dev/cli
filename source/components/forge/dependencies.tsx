import React, {useEffect, useState} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import {runCommandWithOutput} from '../../utils/forge.js';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, packageManager, directory, template} = useForgeStep();
	const [output, setOutput] = useState<string>('');

	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			setOutput('');

			async function installDeps() {
				try {
					const onOutput = (data: string) => {
						setOutput(prev => prev + data);
					};

					if (template?.label.includes('Python')) {
						if (packageManager === 'poetry') {
							await runCommandWithOutput(
								`${packageManager} init`,
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								`${packageManager} env activate`,
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								`${packageManager} install`,
								directory,
								onOutput,
							);
						} else if (packageManager === 'pip') {
							await runCommandWithOutput(
								'python3 -m venv .venv',
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								'source .venv/bin/activate',
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								`${packageManager} install -r requirements.txt`,
								directory,
								onOutput,
							);
						} else if (packageManager === 'uv') {
							await runCommandWithOutput(
								`${packageManager} init`,
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								`${packageManager} venv`,
								directory,
								onOutput,
							);
							await runCommandWithOutput(
								`${packageManager} pip install -r requirements.txt`,
								directory,
								onOutput,
							);
						}
						setLoading(false);
						setTask(true);
						setStep('success');
					} else {
						await runCommandWithOutput(
							`${packageManager} install`,
							directory,
							onOutput,
						);
						setLoading(false);
						setTask(true);
						setStep('success');
					}
				} catch (error) {
					console.error('Error installing dependencies:', error);
					setError('Error installing dependencies');
					setLoading(false);
				}
			}
			installDeps();
		}
	}, [step]);

	return (
		<>
			<Task
				label="Installing dependencies"
				state={state}
				spinner={spinners.dots}
				output={
					state !== 'success'
						? output
								.split('\n')
								.slice(state === 'error' ? -10 : -5)
								.join('\n')
						: undefined
				}
			/>
		</>
	);
}
