import {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {spawnSync} from 'node:child_process';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, directory, template} = useRunStep();

	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			try {
				// Execute all commands from the template's commands array
				if (template?.depCommands && template.depCommands.length > 0) {
					for (const commandStr of template.depCommands) {
						// Parse the command string into command and arguments
						const parts = commandStr.split(' ');
						const command = parts[0] || '';
						const args = parts.slice(1);

						console.log(`Running: ${commandStr}`);

						// Execute the command
						const result = spawnSync(command, args, {
							cwd: directory,
							shell: true,
							stdio: 'inherit',
						});

						if (result.status !== 0) {
							throw new Error(`Command failed: ${commandStr}`);
						}
					}
				}

				setLoading(false);
				setTask(true);
				setStep('runner');
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
