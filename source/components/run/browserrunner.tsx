import React, {useEffect} from 'react';
import {spawn} from 'child_process';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import {CONFIG_DIR, REPO_URL} from '../../utils/constants.js';
import path from 'path';
import spinners from 'cli-spinners';

function isDockerRunning() {
	try {
		spawn('docker', ['info']);
		return true;
	} catch {
		return false;
	}
}

export default function BrowserRunner() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars} = useRunStep();
	useEffect(() => {
		if (step === 'browserrunner' && !task) {
			setLoading(true);
			try {
				spawn('git', ['clone', REPO_URL], {
					cwd: CONFIG_DIR,
				});
				if (!isDockerRunning()) {
					console.log(
						'⚠️ Docker is not running. Please start it and try again.',
					);
					return;
				}
				console.log('🚀 Starting Docker Compose...');
				const folderName = path.basename(REPO_URL, '.git');
				spawn('docker-compose', ['-f', 'docker-compose.dev.yml', 'up', '-d'], {
					cwd: path.join(CONFIG_DIR, folderName),
					stdio: 'inherit',
					env: {
						...process.env,
						API_PORT: String(
							Number(envVars['STEEL_API_URL'].split(':')[2]) ||
								Number(envVars['STEEL_API_URL'].split(':')[1]) ||
								3000,
						),
					},
				});
				setTask('Starting Docker Compose');
				setStep('runner');
				setLoading(false);
			} catch (error) {
				setError(`Error starting Steel Browser Locally ${error}`);
				setLoading(false);
			}
		}
	}, [step]);

	return (
		<Task
			label={`Running Steel Browser locally`}
			state={state}
			spinner={spinners.dots}
		/>
	);
}
