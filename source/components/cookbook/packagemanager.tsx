import React from 'react';
import {Task} from 'ink-task-list';
import SelectInput from 'ink-select-input';
import {useTask} from '../../hooks/usetask.js';
import {useStep} from '../../context/stepcontext.js';
import spinners from 'cli-spinners';

export default function PackageManager() {
	const [state, task, , , setTask] = useTask();

	const {step, setPackageManager, setStep} = useStep();

	return (
		<Task
			label="Pick your package manager"
			state={state}
			spinner={spinners.dots}
			isExpanded={step === 'packagemanager' && !task}
		>
			<SelectInput
				items={[
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
				]}
				onSelect={items => {
					setPackageManager(items.value);
					setTask(items.value);
					setStep('dependencies');
				}}
			/>
		</Task>
	);
}
