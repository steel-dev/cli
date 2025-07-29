import React, {useEffect} from 'react';
import {Task} from 'ink-task-list';
import {TEMPLATES} from '../../utils/constants.js';
import {loadManifest, getTemplateDirectory} from '../../utils/registry.js';
import TemplatePicker from '../templatepicker.js';
import {useTask} from '../../hooks/usetask.js';
import {useRunStep} from '../../context/runstepcontext.js';
import spinners from 'cli-spinners';
import type {Template} from '../../utils/types.js';
import {Args} from '../../commands/run.js';

export default function Template({args}: {args: Args}) {
	const [state, task, , , setTask, setLoading] = useTask();
	const {step, setStep, setTemplate, setDirectory, template} = useRunStep();

	// Bypass selection if args are provided
	useEffect(() => {
		if (step === 'template' && args?.length > 0 && !task) {
			setLoading(true);
			const [templateArg] = args;
			const found = TEMPLATES.find(
				t =>
					t.value === templateArg ||
					t.label === templateArg ||
					t.alias === templateArg ||
					t.command === templateArg,
			);
			if (found) {
				const template = found;
				setTask(true);
				setTemplate(template);

				(async () => {
					try {
						const manifest = loadManifest();
						const example = manifest.examples.find(
							e => e.id === template.value,
						);
						if (example) {
							const dir = await getTemplateDirectory(example, manifest);
							setDirectory(dir);
						}
						setStep('envvar');
						setLoading(false);
					} catch (error) {
						console.error('Failed to set template directory:', error);
						setLoading(false);
					}
				})();
			} else if (templateArg) {
				console.log(`Template "${templateArg}" not found.`);
				setLoading(false);
			}
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

					(async () => {
						try {
							const manifest = loadManifest();
							const example = manifest.examples.find(
								e => e.id === template.value,
							);
							if (example) {
								const dir = await getTemplateDirectory(example, manifest);
								setDirectory(dir);
							}
							setStep('envvar');
							setLoading(false);
						} catch (error) {
							console.error('Failed to set template directory:', error);
						}
					})();
				}}
			/>
		</Task>
	);
}
