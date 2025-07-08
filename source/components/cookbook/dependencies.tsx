import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import {spawnSync} from 'node:child_process';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();

	const {step, setStep, packageManager, directory, template} = useStep();

	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			try {
				if (template?.label.includes('Python')) {
					if (packageManager === 'poetry') {
						spawnSync(packageManager, ['init'], {cwd: directory});
						spawnSync(packageManager, ['env', 'activate'], {cwd: directory});
						spawnSync(packageManager, ['install'], {
							cwd: directory,
						});
					} else if (packageManager === 'pip') {
						spawnSync('python', ['-m', 'venv', '.venv'], {cwd: directory});
						spawnSync('source', ['.venv/bin/activate'], {cwd: directory});
						spawnSync(packageManager, ['install', '-r', 'requirements.txt'], {
							cwd: directory,
						});
					} else if (packageManager === 'uv') {
						spawnSync(packageManager, ['init'], {cwd: directory});
						spawnSync(packageManager, ['venv'], {cwd: directory});
						spawnSync(
							packageManager,
							['pip', 'install', '-r', 'requirements.txt'],
							{
								cwd: directory,
							},
						);
					}
					setLoading(false);
					setTask(true);
					setStep('done');
				} else {
					spawnSync(packageManager, ['install'], {cwd: directory});
					setLoading(false);
					setTask(true);
					setStep('done');
				}
			} catch (error) {
				console.error('Error installing dependencies:', error);
				setError('Error installing dependencies');
				setLoading(false);
			}
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
