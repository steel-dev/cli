import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import {runCommand} from '../../utils/forge.js';
export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, packageManager, directory, template} = useForgeStep();
	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			async function installDeps() {
				try {
					if (template?.label.includes('Python')) {
						if (packageManager === 'poetry') {
							await runCommand(`${packageManager} init`, directory);
							await runCommand(`${packageManager} env activate`, directory);
							await runCommand(`${packageManager} install`, directory);
						} else if (packageManager === 'pip') {
							await runCommand('python3 -m venv .venv', directory);
							await runCommand('source .venv/bin/activate', directory);
							await runCommand(
								`${packageManager} install -r requirements.txt`,
								directory,
							);
						} else if (packageManager === 'uv') {
							await runCommand(`${packageManager} init`, directory);
							await runCommand(`${packageManager} venv`, directory);
							await runCommand(
								`${packageManager} pip install -r requirements.txt`,
								directory,
							);
						}
						setLoading(false);
						setTask(true);
						setStep('runner');
					} else {
						await runCommand(`${packageManager} install`, directory);
						setLoading(false);
						setTask(true);
						setStep('runner');
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
		<Task
			label="Installing dependencies"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
