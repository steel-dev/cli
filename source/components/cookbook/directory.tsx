import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import React, {useEffect} from 'react';
import SelectInput from 'ink-select-input';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import spinners from 'cli-spinners';

export default function Directory({
	step,
	setStep,
	args,
}: {
	step: string;
	setStep: React.Dispatch<React.SetStateAction<string>>;
	args: Array<string>;
}) {
	//@ts-ignore
	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();

	useEffect(() => {
		let timer: NodeJS.Timeout;
		if (step === 'directory' && task) {
			setLoading(true);
			timer = setTimeout(() => {
				setLoading(false);
				setStep('scaffold');
			}, 3500);
		}
		return () => clearTimeout(timer);
	}, [step, task]);

	const renameFiles: Record<string, string | undefined> = {
		_gitignore: '.gitignore',
	};

	let targetDir = args[0];

	if (!targetDir) {
		targetDir = 'steel-project';
	}

	const templateDir = path.resolve(
		fileURLToPath(import.meta.url),
		'../../examples',
		selectedTemplate?.value,
	);

	const write = (file: string, content?: string) => {
		const targetPath = path.join(root, renameFiles[file] ?? file);
		if (content) {
			fs.writeFileSync(targetPath, content);
		} else {
			copy(path.join(templateDir, file), targetPath);
		}
	};

	const files = fs.readdirSync(templateDir);
	for (const file of files.filter(f => f !== 'package.json')) {
		write(file);
	}

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
