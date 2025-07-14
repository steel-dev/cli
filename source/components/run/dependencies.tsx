import {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {runCommand} from '../../utils/forge.js';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, directory, template} = useRunStep();

	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);

			async function installDeps() {
				try {
					if (template?.depCommands && template.depCommands.length > 0) {
						const commandStr = template.depCommands.join(' && ');
						await runCommand(commandStr, [], directory);
					}
					setTask(true);
					setLoading(false);

					if (envVars['STEEL_API_URL']) {
						setStep('browserrunner');
					} else {
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
	}, [step, task]);

	return (
		<Task
			label="Installing dependencies"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
