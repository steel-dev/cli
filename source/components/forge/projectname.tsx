import React, {useState, useEffect} from 'react';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import {toValidProjectName} from '../../utils/forge.js';
import TextInput from 'ink-text-input';
import {Options} from '../../commands/forge.js';

export default function ProjectName({options}: {options: Options}) {
	const {step, setStep, template, setDirectory} = useForgeStep();
	const [state, task, , , setTask] = useTask();
	const [query, setQuery] = useState('');

	useEffect(() => {
		if (step === 'projectname' && !task && options.name) {
			setTask(toValidProjectName(options.name));
			setDirectory(toValidProjectName(options.name));
			setStep('packagemanager');
		}
	}, [step, task]);

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
				placeholder={toValidProjectName(template.value)}
				onSubmit={() => {
					if (query) {
						setTask(toValidProjectName(query));
						setDirectory(toValidProjectName(query));
					} else {
						setTask(toValidProjectName(template.value));
						setDirectory(toValidProjectName(template.value));
					}
					setStep('packagemanager');
				}}
			/>
		</Task>
	);
}
