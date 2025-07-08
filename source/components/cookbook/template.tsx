import React from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import {Template} from '../../utils/types.js';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function Template() {
	const [state, task, , , setTask, ,] = useTask();
	const {step, setStep, setTemplate} = useStep();
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
					setTemplate(items);
					setStep('packagemanager');
				}}
			/>
		</Task>
	);
}
