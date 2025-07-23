import path from 'path';
import fs from 'fs';
import React from 'react';
import {useEffect, useState} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {runCommand} from '../../utils/forge.js';
import {hashDeps} from '../../utils/cache.js';
import {CACHE_DIR} from '../../utils/constants.js';

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, directory, template, setHash} = useRunStep();
	const [output, setOutput] = useState<string>('');

	// Check if the template has a TASK environment variable
	const hasTaskEnvVar = template?.env?.some(e => e.value === 'TASK');

	// Helper function to determine next step
	const getNextStep = () => {
		if (hasTaskEnvVar) {
			return 'task';
		}
		if (
			envVars['STEEL_API_URL'] &&
			!envVars['STEEL_API_URL'].includes('api.steel.dev')
		) {
			return 'browserrunner';
		} else {
			return 'runner';
		}
	};

	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			async function installDeps() {
				try {
					if (template?.depCommands && directory) {
						const depFileName =
							template.language === 'PY' ? 'requirements.txt' : 'package.json';
						const depFilePath = path.join(directory, depFileName);

						if (fs.existsSync(depFilePath)) {
							const hash = hashDeps(depFilePath);
							setHash(hash);
							const depsDir = path.join(CACHE_DIR, hash);
							const depCommands = template.depCommands({
								depsDir,
							});
							if (!fs.existsSync(depsDir)) {
								for (const command of depCommands) {
									await runCommand(command, directory);
								}
							}
						}
					}
					setStep(getNextStep());
					setTask(true);
					setLoading(false);
				} catch (error) {
					console.error('Error installing dependencies:', error);
					setError(`Error installing dependencies: ${error.message}`);
					setOutput(`Error installing dependencies: ${error.message}`);
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
			output={output}
		/>
	);
}
