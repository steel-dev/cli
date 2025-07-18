import path from 'path';
import fs from 'fs';
import React from 'react';
import {fileURLToPath} from 'url';
import {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import {runCommand} from '../../utils/forge.js';
import {hashDeps} from '../../utils/cache.js';
import {CACHE_DIR} from '../../utils/constants.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default function Dependencies() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, envVars, directory, template, setHash} = useRunStep();
	useEffect(() => {
		if (step === 'dependencies' && !task) {
			setLoading(true);
			async function installDeps() {
				try {
					if (template?.depCommands) {
						// Takes the dependencies file (requirements.txt/package.json) and hashes it to create a unique identifier for caching purposes
						const hash = hashDeps(
							path.resolve(
								__dirname,
								`../../examples/${template.value}/${template.label.includes('Python') ? 'requirements.txt' : 'package.json'}`,
							),
						);
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
					if (envVars['STEEL_API_URL']) {
						setStep('browserrunner');
					} else {
						setStep('runner');
					}
					setTask(true);
					setLoading(false);
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
