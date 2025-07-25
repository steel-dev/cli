import path from 'path';
import React, {useEffect} from 'react';
import {fileURLToPath} from 'url';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import TemplatePicker from '../templatepicker.js';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import type {Template} from '../../utils/types.js';
import {Args} from '../../commands/run.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default function Template({args}: {args: Args}) {
	const [state, task, , , setTask, setLoading] = useTask();
	const {step, setStep, setTemplate, setDirectory, template} = useRunStep();

	// Bypass selection if args are provided
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
				setDirectory(
					path.resolve(__dirname, `../../examples/${template.value}/`),
				);
				setStep('envvar');
			} else if (templateArg) {
				console.log(`Template "${templateArg}" not found.`);
				// Optionally: fall back to selection or exit
			}
			setLoading(false);
		}
	}, [args, task]);

	// Compute label based on current state
	const label = template
		? `You selected: \x1b[35m${template.alias} (${template.language})\x1b[0m`
		: 'Select template';

	// Skip UI if task is already set (from args)
	// Return complete task for viewer
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
					setDirectory(
						path.resolve(__dirname, `../../examples/${template.value}/`),
					);
					setStep('envvar');
				}}
			/>
		</Task>
	);
}
