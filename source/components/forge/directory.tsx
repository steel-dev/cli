import fs from 'fs';
import path from 'path';
import React from 'react';
import {fileURLToPath} from 'url';
import {useEffect} from 'react';
import SelectInput from 'ink-select-input';
import {Task} from 'ink-task-list';
import {write} from '../../utils/forge.js';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';

export default function Directory() {
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const {step, setStep, directory, template} = useForgeStep();

	useEffect(() => {
		if (step === 'directory') {
			if (!template) {
				setLoading(false);
				setStep('template');
				return;
			}

			// Check if directory exists
			const directoryExists = fs.existsSync(directory);

			// If directory doesn't exist, proceed automatically without showing options
			if (!directoryExists) {
				setTask('proceed'); // Set task to proceed automatically
				return;
			}

			// If directory exists but no task is set, show the selection options
			if (!task) {
				return; // This will show the SelectInput component
			}
		}
	}, [step, template, directory, task, setLoading, setStep, setTask]);

	useEffect(() => {
		if (step === 'directory' && task) {
			setLoading(true);
			const cwd = process.cwd();

			const templateDir = path.resolve(
				fileURLToPath(import.meta.url),
				'../../../../examples',
				template?.value,
			);
			const files = fs.readdirSync(templateDir);

			// Create directory if it doesn't exist, or handle existing directory based on task
			if (task === 'proceed' || task === 'yes' || task === 'ignore') {
				if (task === 'yes') {
					// Remove existing files if user chose to
					if (fs.existsSync(directory)) {
						fs.rmSync(directory, {recursive: true, force: true});
					}
				}

				// Create directory (this will work whether it exists or not)
				fs.mkdir(directory, {recursive: true}, err => {
					if (err && err.code !== 'EEXIST') {
						setError(err?.message);
						return;
					}
				});

				// Copy files
				for (const file of files.filter(f => f !== 'package.json')) {
					write(file, directory, templateDir, cwd);
				}

				const projectName = path.basename(path.resolve(directory));
				const packageJsonPath = path.join(templateDir, `package.json`);
				if (fs.existsSync(packageJsonPath)) {
					const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
					pkg.name = projectName;
					write(
						'package.json',
						directory,
						templateDir,
						cwd,
						JSON.stringify(pkg, null, 2) + '\n',
					);
				}
			}

			setLoading(false);
			setStep('envvar');
		}
	}, [step, task, directory, template, setLoading, setError, setStep]);

	const shouldShowSelection =
		step === 'directory' && !task && fs.existsSync(directory);

	return (
		<Task
			label="Writing directory"
			state={state}
			spinner={spinners.dots}
			isExpanded={shouldShowSelection}
		>
			{shouldShowSelection && (
				<SelectInput
					items={[
						{
							label: 'Remove existing files and continue',
							value: 'yes',
						},
						{
							label: 'Ignore files and continue',
							value: 'ignore',
						},
					]}
					onSelect={items => setTask(items.value)}
				/>
			)}
		</Task>
	);
}
