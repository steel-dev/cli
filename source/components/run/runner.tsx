import {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {spawn} from 'child_process';

export default function Runner() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, template, directory} = useRunStep();
	//@ts-ignore
	useEffect(() => {
		if (step === 'runner' && !task) {
			setLoading(true);

			if (!template?.runCommand) {
				setError('No run command specified in template');
				setLoading(false);
			}

			try {
				// Parse the command into the binary and its arguments
				const parts = template.runCommand ? template.runCommand.split(' ') : [];
				const command = parts[0] || '';
				const args = parts.slice(1);

				spawn(command, args, {
					cwd: directory,
					shell: true,
					stdio: 'inherit', // This will pipe stdout and stderr to the parent process
				});

				setTask(`Command completed successfully: ${template.runCommand}`);
				setStep('browser');
				setLoading(false);
			} catch (error) {
				console.error('Error running command:', error);
				setError(`Error running command: ${(error as Error).message}`);
				setLoading(false);
			}
		}
	}, [step, template]);

	return (
		<Task
			label={`Running ${template?.label + ' ' || ''}example`}
			state={state}
			spinner={spinners.dots}
		/>
	);
}
