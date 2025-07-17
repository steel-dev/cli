#!/usr/bin/env node

import React from 'react';
import Template from '../components/run/template.js';
import Runner from '../components/run/runner.js';
import EnvVar from '../components/run/envvar.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/run/dependencies.js';
import {RunStepProvider} from '../context/runstepcontext.js';
import BrowserOpener from '../components/run/browseropener.js';
import BrowserRunner from '../components/run/browserrunner.js';
import zod from 'zod';
import {option} from 'pastel';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example Project to run').optional(),
]);

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
				description: 'Auto open live session viewer',
				alias: 'v',
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
});

export type Options = zod.infer<typeof options>;

export type Args = zod.infer<typeof args>;

type Props = {
	args: Args;
	options: Options;
};

export default function Cookbook({args, options}: Props) {
	return (
		<RunStepProvider>
			<TaskList>
				<Template args={args} />
				<EnvVar options={options} />
				<Dependencies />
				<BrowserRunner />
				<Runner options={options} />
				<BrowserOpener options={options} />
			</TaskList>
		</RunStepProvider>
	);
}
