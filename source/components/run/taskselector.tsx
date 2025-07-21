import React, {useEffect, useState} from 'react';
import {Box, Text} from 'ink';
import TextInput from 'ink-text-input';
import {Task} from 'ink-task-list';
import spinners from 'cli-spinners';
import {useRunStep} from '../../context/runstepcontext.js';
import {useTask} from '../../hooks/usetask.js';
import type {Options} from '../../commands/run.js';

export default function TaskSelector({options}: {options: Options}) {
	const {step, setStep, template, envVars, setEnvVars} = useRunStep();
	const [state, task, , , setTask, setLoading, setError] = useTask();
	const [inputValue, setInputValue] = useState('');
	const [isCollectingTask, setIsCollectingTask] = useState(false);

	// Check if the template has a TASK environment variable
	const hasTaskEnvVar = template?.env?.some(e => e.value === 'TASK');

	// Helper function to determine next step
	const getNextStep = () => {
		if (
			envVars['STEEL_API_URL'] &&
			!envVars['STEEL_API_URL'].includes('api.steel.dev')
		) {
			return 'browserrunner';
		} else {
			return 'runner';
		}
	};

	useEffect(() => {
		if (step === 'task' && !task && !isCollectingTask) {
			setLoading(true);
			try {
				// Check if task was already provided via options
				if (options.task) {
					const updatedEnvVars = {
						...envVars,
						TASK: options.task,
					};
					setEnvVars(updatedEnvVars);
					setTask(updatedEnvVars);
					setStep(getNextStep());
				} else if (hasTaskEnvVar) {
					// Need to collect task from user
					setIsCollectingTask(true);
				} else {
					// No task needed, proceed to next step
					setTask(envVars);
					setStep(getNextStep());
				}
				setLoading(false);
			} catch (error) {
				console.error('Error setting up task:', error);
				setError('Error setting up task');
				setLoading(false);
			}
		}
	}, [
		step,
		task,
		isCollectingTask,
		options.task,
		hasTaskEnvVar,
		envVars,
		setEnvVars,
		setTask,
		setStep,
		setLoading,
		setError,
	]);

	const handleInputSubmit = (val: string) => {
		const updatedEnvVars = {
			...envVars,
			TASK: val,
		};
		setEnvVars(updatedEnvVars);
		setIsCollectingTask(false);
		setTask(updatedEnvVars);
		setStep(getNextStep());
		setInputValue('');
	};

	// Only render if this step is active and template has TASK env var
	if (!hasTaskEnvVar) {
		return null;
	}

	return (
		<Box flexDirection="column">
			<Task
				label="Setting up task"
				state={state}
				spinner={spinners.dots}
				isExpanded={step === 'task' && !task && isCollectingTask}
			>
				{isCollectingTask && (
					<>
						<Text>
							🎯 Enter the task for the agent:
							<Text color="cyan"> What should the agent do?</Text>
						</Text>
						<TextInput
							value={inputValue}
							onChange={setInputValue}
							onSubmit={handleInputSubmit}
							placeholder="Describe the task you want the agent to perform..."
						/>
					</>
				)}
			</Task>
		</Box>
	);
}
