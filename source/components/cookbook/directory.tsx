import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import React, {useEffect} from 'react';
import SelectInput from 'ink-select-input';
import {Task} from 'ink-task-list';
import {write, isValidProjectName} from '../../utils/cookbook.js';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function Directory({args}: {args: Array<string>}) {
	//@ts-ignore
	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();

	const {step, setStep, setDirectory, template} = useStep();

	let targetDir = args[0];

	if (!targetDir) {
		targetDir = 'steel-project';
	}

	setDirectory(targetDir);

	useEffect(() => {
		if (step === 'directory' && task) {
			setLoading(true);

			const cwd = process.cwd();

			if (!template) {
				return;
			}

			const templateDir = path.resolve(
				fileURLToPath(import.meta.url),
				'../../examples',
				template?.value,
			);

			const files = fs.readdirSync(templateDir);
			for (const file of files.filter(f => f !== 'package.json')) {
				write(file, targetDir, templateDir, cwd);
			}

			let projectName = path.basename(path.resolve(targetDir));
			if (!isValidProjectName(projectName)) {
				setLoading(false);
				setStep('projectname');
			} else {
				const packageJsonPath = path.join(templateDir, `package.json`);
				if (fs.existsSync(packageJsonPath)) {
					const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
					pkg.name = projectName;
					write(
						'package.json',
						targetDir,
						templateDir,
						cwd,
						JSON.stringify(pkg, null, 2) + '\n',
					);
				}
				setStep('scaffold');
			}
		}
	}, [step, task]);

	function isEmpty(path: string) {
		const files = fs.readdirSync(path);
		return files.length === 0 || (files.length === 1 && files[0] === '.git');
	}

	return (
		<Task
			label="Checking direcory"
			state={state}
			spinner={spinners.dots}
			isExpanded={
				step === 'directory' &&
				!task &&
				fs.existsSync(targetDir) &&
				!isEmpty(targetDir)
			}
		>
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
		</Task>
	);
}
