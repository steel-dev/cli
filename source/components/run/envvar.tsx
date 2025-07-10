import path from 'path';
import {useEffect} from 'react';
import {getApiKey} from '../../utils/session.js';
import {updateEnvVariable} from '../../utils/cookbook.js';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import fs from 'fs';
import {v4 as uuidv4} from 'uuid';
import {ENV_VAR_MAP} from '../../utils/constants.js';

export default function EnvVar({options}: {options: any}) {
	const {step, setStep, directory} = useRunStep();
	const [state, , , , setTask, setLoading, setError] = useTask();
	const workingDir = directory || process.cwd();

	useEffect(() => {
		if (step === 'envvar') {
			setLoading(true);
			try {
				const apiKey = getApiKey();
				if (apiKey) {
					setTask(apiKey);
				} else {
					setTask('your-api-key-here');
				}

				// Process all env variables from options
				const envPath = path.join(workingDir, '.env.example');
				if (fs.existsSync(envPath)) {
					fs.copyFileSync(envPath, path.join(workingDir, '.env'));

					// Update all env variables
					for (const [key, envVar] of Object.entries(ENV_VAR_MAP)) {
						if (key in options) {
							updateEnvVariable(workingDir, envVar, String(options[key]));
							if (key === 'api-url') {
								updateEnvVariable(
									workingDir,
									'CONNECT_URL',
									String('ws:' + options[key].split(':')[1]),
								);
							}
						}
					}
					if (!('api-url' in options)) {
						updateEnvVariable(workingDir, 'STEEL_SESSION_ID', uuidv4());
					}
					fs.unlinkSync(envPath);
				}
				setLoading(false);
				setStep('dependencies');
			} catch (error) {
				console.error('Error updating environment variables:', error);
				setError('Error updating environment variables');
				setLoading(false);
			}
		}
	}, [step]);

	return (
		<Task
			label="Setting up environment variables"
			state={state}
			spinner={spinners.dots}
		/>
	);
}
