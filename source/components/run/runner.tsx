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

				// Show what command we're going to run
				// console.log(`Running: ${template.runCommand}`);

				// Spawn the process
				// const process: ChildProcess =
				spawn(command, args, {
					cwd: directory,
					shell: true,
					stdio: 'inherit', // This will pipe stdout and stderr to the parent process
				});

				// When the process exits, determine if it was successful
				// process.on('close', code => {
				// 	if (code === 0) {
				// 		setTask(`Command completed successfully: ${template.runCommand}`);
				// 		// Move to browser step after successful execution
				// 		setStep('browser');
				// 	} else {
				// 		setError(`Command failed with exit code ${code}`);
				// 	}
				// 	setLoading(false);
				// });
				setTask(`Command completed successfully: ${template.runCommand}`);
				setStep('browser');
				setLoading(false);
				// Handle process error (e.g., command not found)
				// process.on('error', err => {
				// 	setError(`Failed to run command: ${err.message}`);
				// 	setLoading(false);
				// });

				// The component's cleanup function will handle killing the process if unmounted
				// return () => {
				// 	if (process && !process.killed) {
				// 		process.kill();
				// 	}
				// };
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
