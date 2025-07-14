import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import {useEffect} from 'react';
import SelectInput from 'ink-select-input';
import {Task} from 'ink-task-list';
import {write} from '../../utils/forge.js';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';

export default function Directory() {
	const [state, task, , , setTask, setLoading, setError] = useTask();

	const {step, setStep, directory, template} = useRunStep();

	useEffect(() => {
		function buildDirectory() {
			// let timer: NodeJS.Timeout;
			if (step === 'directory' && task) {
				setLoading(true);

				const cwd = process.cwd();

				if (!template) {
					setLoading(false);
					setStep('template');
					return;
				}

				const templateDir = path.resolve(
					fileURLToPath(import.meta.url),
					'../../../../examples',
					template?.value,
				);

				const files = fs.readdirSync(templateDir);
				fs.mkdir(directory, err => setError(err?.message));
				for (const file of files.filter(f => f !== 'package.json')) {
					write(file, directory, templateDir, cwd);
				}

				let projectName = path.basename(path.resolve(directory));
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
				// timer = setTimeout(() => {
				setLoading(false);
				setStep('envvar');
				// }, 6000);
			}
		}
		buildDirectory();
		// return () => clearTimeout(timer);
	}, [step, task]);

	return (
		<Task
			label="Writing directory"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'directory' && !task}
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
