import React, {useState} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import {toValidProjectName} from '../../utils/forge.js';
import TextInput from 'ink-text-input';
import {Args} from '../../commands/forge.js';

export default function ProjectName({args}: {args: Args}) {
	const {step, setStep, setDirectory} = useForgeStep();
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
				placeholder={args[0] || 'steel-starter'}
				onSubmit={() => {
					if (!query && args[0]) {
						setTask(toValidProjectName(args[0]));
						setDirectory(toValidProjectName(args[0]));
					} else if (query) {
						setTask(toValidProjectName(query));
						setDirectory(toValidProjectName(query));
					} else {
						setTask('steel-starter');
						setDirectory('steel-starter');
					}
					setStep('packagemanager');
				}}
			/>
		</Task>
	);
}
