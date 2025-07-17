import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import type {Template} from '../../utils/types.js';
import {Args} from '../../commands/forge.js';

export default function Template({args}: {args: Args}) {
	const [state, task, , , setTask, setLoading] = useTask();
	const {step, setStep, setTemplate} = useForgeStep();

	useEffect(() => {
		if (step === 'template' && args?.length > 0 && !task) {
			setLoading(true);
			const [templateArg] = args;
			// Find a matching template by label/key/etc.
			const found = TEMPLATES.find(
				t =>
					t.value === templateArg ||
					t.label === templateArg ||
					t.alias === templateArg,
			);
			if (found) {
				const template = found;
				setTask(template);
				setTemplate(template);
				setStep('packagemanager');
			} else if (templateArg) {
				console.log(`Template "${templateArg}" not found.`);
				// Optionally: fall back to selection or exit
			}
			setLoading(false);
		}
	}, [args, task]);

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
					setStep('projectname');
				}}
			/>
		</Task>
	);
}
