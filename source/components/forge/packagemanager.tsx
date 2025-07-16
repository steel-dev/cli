import React from 'react';
import {Task} from 'ink-task-list';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useForgeStep} from '../../context/forgestepcontext.js';
import spinners from 'cli-spinners';
export default function PackageManager() {
	const [state, task, , , setTask] = useTask();
	const {step, setPackageManager, setStep, template} = useForgeStep();
	const items = template?.label.includes('Python')
		? [
				{label: 'pip', value: 'pip'},
				{label: 'poetry', value: 'poetry'},
				{label: 'uv', value: 'uv'},
			]
		: [
				{
					label: 'npm',
					value: 'npm',
				},
				{
					label: 'yarn',
					value: 'yarn',
				},
				{
					label: 'pnpm',
					value: 'pnpm',
				},
				{
					label: 'bun',
					value: 'bun',
				},
			];

	return (
		<Task
			label="Pick your package manager"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'packagemanager' && !task}
		>
			<SelectInput
				items={items}
				onSelect={items => {
					setPackageManager(items.value);
					setTask(items.value);
					setStep('directory');
				}}
			/>
		</Task>
	);
}
