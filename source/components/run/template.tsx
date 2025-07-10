import {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import {Template} from '../../utils/types.js';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';

export default function Template({args}: {args?: any}) {
	const [state, task, , , setTask, ,] = useTask();
	const {step, setStep, template, setTemplate, setDirectory} = useRunStep();
	// Bypass selection if args are provided
	useEffect(() => {
		if (args?.length) {
			const [templateArg] = args;

			// Find a matching template by label/key/etc.
			const found = TEMPLATES.find(
				t =>
					t.value === templateArg ||
					t.label === templateArg ||
					t.alias === templateArg,
			);

			if (found) {
				const template = found as Template;
				setTask(template);
				setTemplate(template);
				setDirectory(template.value);
				setStep('directory');
			} else if (templateArg) {
				console.log(`Template "${templateArg}" not found.`);
				// Optionally: fall back to selection or exit
			}
		}
	}, [args, setTask, setTemplate, setStep]);

	// Skip UI if task is already set (from args)
	// Return complete task for viewer
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
					setDirectory(template.value);
					setStep('directory');
				}}
			/>
		</Task>
	);
}
