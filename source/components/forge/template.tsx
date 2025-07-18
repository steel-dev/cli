import React, {useEffect, useState} from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import TemplatePicker from '../templatepicker.js';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
import type {Template} from '../../utils/types.js';
import {Args} from '../../commands/forge.js';
import {Text} from 'ink';

export default function Template({args}: {args: Args}) {
	const [state, task, , , setTask, setLoading] = useTask();
	const {step, setStep, setTemplate} = useForgeStep();
	const [label, setLabel] = useState('Select template');

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
				setTask(true);
				setTemplate(template);
				setLabel(`You selected: ${template.alias} (${template.language})`);
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
			label={label}
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'template' && !task}
		>
			<TemplatePicker
				templates={TEMPLATES}
				onSelect={template => {
					setTask(true);
					setTemplate(template);
					setLabel(
						`You selected: ` +
						// magenta color for alias and language
						// (Ink <Text> not available in setLabel, so use ANSI escape codes)
						`\x1b[35m${template.alias} (${template.language})\x1b[0m`
					);
					setStep('projectname');
				}}
			/>
		</Task>
	);
}
