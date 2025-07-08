import React, {useState} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';
import {toValidProjectName} from '../../utils/cookbook.js';
import TextInput from 'ink-text-input';

export default function ProjectName({args}: {args: Array<string>}) {
	const {step, setStep, setDirectory} = useStep();
	const [state, task, , , setTask] = useTask();
	const [query, setQuery] = useState('');

	return (
		<Task
			label="Project Name"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'projectname' && !task}
		>
			<TextInput
				value={query}
				onChange={setQuery}
				placeholder={args[0]}
				onSubmit={() => {
					if (!query && args[0]) {
						setTask(toValidProjectName(args[0]));
						setDirectory(toValidProjectName(args[0]));
					} else if (query) {
						setTask(toValidProjectName(query));
						setDirectory(toValidProjectName(query));
					}
					setStep('template');
				}}
			/>
		</Task>
	);
}
