import React from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import {Template} from '../../utils/types.js';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import spinners from 'cli-spinners';

export default function Template({
	step,
	setStep,
}: {
	step: string;
	setStep: React.Dispatch<React.SetStateAction<string>>;
}) {
	const [state, task, , , setTask, ,] = useTask();
	return (
		<Task
			label="Select template"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'template' && !task}
		>
			<SelectInput
				items={TEMPLATES}
				onSelect={items => {
					setTask(items);
					setStep('directory');
				}}
			/>
		</Task>
	);
}
