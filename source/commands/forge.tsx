#!/usr/bin/env node

import React from 'react';
import EnvVar from '../components/forge/envvar.js';
import Template from '../components/forge/template.js';
import Directory from '../components/forge/directory.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/forge/dependencies.js';
import ProjectName from '../components/forge/projectname.js';
import {ForgeStepProvider, useForgeStep} from '../context/forgestepcontext.js';
import CLIWelcomeMessage from '../components/cliwelcomemessage.js';
import zod from 'zod';
import {option} from 'pastel';
import PackageManager from '../components/forge/packagemanager.js';
import ForgeSuccess from '../components/forge/success.js';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example template to start').optional(),
]);

export const argsLabels = ['template'];

export const options = zod.object({
	name: zod
		.string()
		.describe(
			option({
				description: 'Name of the project',
				alias: 'n',
			}),
		)
		.optional(),
	api_url: zod
		.string()
		.describe(
			option({
				description: 'API URL for Steel API',
				alias: 'a',
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
});

export type Options = zod.infer<typeof options>;
export type Args = zod.infer<typeof args>;

type Props = {
	args: Args;
	options: Options;
};

function getStepOrder(): string[] {
	const baseSteps = [
		'template',
		'projectname',
		'packagemanager',
		'directory',
		'envvar',
		'dependencies',
		'success',
	];

	return baseSteps;
}

function shouldShowStep(targetStep: string, currentStep: string): boolean {
	const stepOrder = getStepOrder();
	const currentIndex = stepOrder.indexOf(currentStep);
	const targetIndex = stepOrder.indexOf(targetStep);

	return targetIndex <= currentIndex;
}

function ForgeContent({args, options}: Props) {
	const {step} = useForgeStep();

	return (
		<>
			<CLIWelcomeMessage />
			<TaskList>
				{shouldShowStep('template', step) && <Template args={args} />}
				{shouldShowStep('projectname', step) && <ProjectName args={args} />}
				{shouldShowStep('packagemanager', step) && <PackageManager />}
				{shouldShowStep('directory', step) && <Directory />}
				{shouldShowStep('envvar', step) && <EnvVar options={options} />}
				{shouldShowStep('dependencies', step) && <Dependencies />}
				{shouldShowStep('success', step) && <ForgeSuccess />}
			</TaskList>
		</>
	);
}

export default function Forge({args, options}: Props) {
	return (
		<ForgeStepProvider>
			<ForgeContent args={args} options={options} />
		</ForgeStepProvider>
	);
}
