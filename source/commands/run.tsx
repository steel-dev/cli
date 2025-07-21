#!/usr/bin/env node

import React from 'react';
import Template from '../components/run/template.js';
import Runner from '../components/run/runner.js';
import EnvVar from '../components/run/envvar.js';
import TaskSelector from '../components/run/taskselector.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/run/dependencies.js';
import {RunStepProvider, useRunStep} from '../context/runstepcontext.js';
import BrowserRunner from '../components/run/browserrunner.js';
import CLIWelcomeMessage from '../components/cliwelcomemessage.js';
import zod from 'zod';
import {option} from 'pastel';
import {getSettings} from '../utils/session.js';
import type {Template as TemplateType} from '../utils/types.js';

export const description =
	'Run a Steel Cookbook automation instantly from the CLI — no setup, no files.';

export const args = zod.tuple([
	zod.string().describe('Example template to run').optional(),
]);

export const argsLabels = ['template'];

export const options = zod.object({
	api_url: zod
		.string()
		.describe(
			option({
				description: 'API URL for Steel API',
				alias: 'a',
			}),
		)
		.optional(),
	view: zod
		.boolean()
		.describe(
			option({
				description: 'Open live session viewer',
				alias: 'o',
			}),
		)
		.optional(),
	task: zod
		.string()
		.describe(
			option({
				description: 'Task to run',
				alias: 't',
			}),
		)
		.optional(),
	api_key: zod
		.string()
		.describe(
			option({
				description: 'API Key for Steel API',
			}),
		)
		.optional(),
	openai_key: zod
		.string()
		.describe(option({description: 'API Key for OpenAI'}))
		.optional(),
	skip_auth: zod.boolean().describe('Skip authentication').optional(),
	help: zod
		.boolean()
		.describe(option({description: 'Show help', alias: 'h'}))
		.optional(),
});

export type Options = zod.infer<typeof options>;

export type Args = zod.infer<typeof args>;

type Props = {
	args: Args;
	options: Options;
};

// Helper function to determine step order and visibility
function getStepOrder(
	template: TemplateType | null,
	envVars: Record<string, string> | null,
): string[] {
	const baseSteps = ['template', 'envvar', 'dependencies'];

	// Add task step if template has TASK environment variable
	const hasTaskEnvVar = template?.env?.some(
		(e: {value: string; label: string; required?: boolean}) =>
			e.value === 'TASK',
	);
	if (hasTaskEnvVar) {
		baseSteps.push('task');
	}

	// Add browserrunner or runner based on API URL
	if (
		envVars?.['STEEL_API_URL'] &&
		!envVars['STEEL_API_URL'].includes('api.steel.dev')
	) {
		baseSteps.push('browserrunner');
	}

	baseSteps.push('runner');
	baseSteps.push('browser', 'done');

	return baseSteps;
}

function shouldShowTask(
	taskStep: string,
	currentStep: string,
	template: TemplateType | null,
	envVars: Record<string, string> | null,
): boolean {
	const stepOrder = getStepOrder(template, envVars);
	const currentIndex = stepOrder.indexOf(currentStep);
	const taskIndex = stepOrder.indexOf(taskStep);

	// Show if current step or if step has been passed (completed)
	return taskIndex <= currentIndex;
}

function RunContent({args, options}: Props) {
	const {step, template, envVars} = useRunStep();
	const settings = getSettings();

	return (
		<>
			<CLIWelcomeMessage />
			<TaskList>
				{shouldShowTask('template', step, template, envVars) && (
					<Template args={args} />
				)}
				{shouldShowTask('envvar', step, template, envVars) && (
					<EnvVar options={options} />
				)}
				{shouldShowTask('dependencies', step, template, envVars) && (
					<Dependencies />
				)}
				{shouldShowTask('task', step, template, envVars) && (
					<TaskSelector options={options} />
				)}
				{shouldShowTask('browserrunner', step, template, envVars) &&
					settings.instance === 'local' && <BrowserRunner />}
				{shouldShowTask('runner', step, template, envVars) && (
					<Runner options={options} />
				)}
			</TaskList>
		</>
	);
}

export default function Run({args, options}: Props) {
	return (
		<RunStepProvider>
			<RunContent args={args} options={options} />
		</RunStepProvider>
	);
}
