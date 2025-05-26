import path from 'path';
import React from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import {toValidProjectName} from '../../utils/cookbook.js';
import TextInput from 'ink-text-input';

export default function ProjectName({args}: {args: Array<string>}) {
	const {setStep, setDirectory} = useStep();
	const [state, task, , , setTask] = useTask();

	return (
		<Task label="Project Name" state={state} spinner={spinners.dots}>
			<TextInput
				value={task}
				onChange={setTask}
				placeholder={args[0]}
				onSubmit={() => {
					setTask(toValidProjectName(task));
					setDirectory(path.join(process.cwd(), toValidProjectName(task)));
					setStep('template');
				}}
			/>
		</Task>
	);
}
