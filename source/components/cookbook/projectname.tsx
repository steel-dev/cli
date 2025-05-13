import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import {write, isValidProjectName} from '../../utils/cookbook.js';
import TextInput from 'ink-text-input';

export default function ProjectName({args}: {args: Array<string>}) {
	const {step, setStep, template} = useStep();
	const [state, task, , , setTask, setLoading] = useTask();
	useEffect(() => {
		if (step === 'projectname') {
		}
	}, [step]);

	return (
		<Task label="Project Name" state={state} spinner={spinners.dots}>
			<TextInput
				value={task}
				onChange={setTask}
				onSubmit={() => {
					if (isValidProjectName(task)) {
						setLoading(true);
						let targetDir = args[0];

						if (!targetDir) {
							targetDir = 'steel-project';
						}

						const cwd = process.cwd();

						if (!template) {
							return;
						}

						const templateDir = path.resolve(
							fileURLToPath(import.meta.url),
							'../../examples',
							template?.value,
						);
						const packageJsonPath = path.join(templateDir, `package.json`);
						if (fs.existsSync(packageJsonPath)) {
							const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
							pkg.name = task;
							write(
								'package.json',
								targetDir,
								templateDir,
								cwd,
								JSON.stringify(pkg, null, 2) + '\n',
							);
						}
						setLoading(false);
						setStep('scaffold');
					}
				}}
			/>
		</Task>
	);
}
