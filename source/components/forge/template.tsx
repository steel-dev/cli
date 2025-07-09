import { useEffect } from 'react';}
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import {Template} from '../../utils/types.js';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function Template({args}: {args?: Array<string>}) {
	const [state, task, , , setTask, ,] = useTask();
	const {step, setStep, template, setTemplate} = useStep();

	useEffect(() => {
		if (!args?.length) return;

		const [templateArg] = args;

		const found = TEMPLATES.find(
			t => t.value === templateArg || t.label === templateArg,
		);

		if (found) {
			const template = found as Template;
			setTask(template);
			setTemplate(template);
			setStep('packagemanager');
		} else {
			console.warn(
				`Template "${templateArg}" not found. Falling back to manual selection.`,
			);
		}
	}, [args, setTask, setTemplate, setStep]);

	// Skip UI if task is already set (from args)
	if (task || template) return null;
	return (
		<Task
			label="Select template"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'template' && !task}
		>
			<SelectInput
				items={TEMPLATES}
				onSelect={item => {
					const template = item as Template;
					setTask(template);
					setTemplate(template);
					setStep('packagemanager');
				}}
			/>
		</Task>
	);
}
