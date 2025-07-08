import React from 'react';
import SteelApiKey from '../components/cookbook/steelapikey.js';
import Template from '../components/cookbook/template.js';
import Directory from '../components/cookbook/directory.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/cookbook/dependencies.js';
import ProjectName from '../components/cookbook/projectname.js';
import {StepProvider} from '../context/stepcontext.js';
import zod from 'zod';
import {option} from 'pastel';
import PackageManager from '../components/cookbook/packagemanager.js';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example Project to run'),
]);

export const options = zod.object({
	'base-url': zod.string().describe(
		option({
			description: 'Base URL for Steel API',
			alias: 'b',
		}),
	),
	view: zod.string().describe(
		option({
			description: 'Auto open live session viewer',
			alias: 'v',
		}),
	),
});

type Props = {
	args: zod.infer<typeof args>;
	options: zod.infer<typeof options>;
};

export default function Cookbook({args, options}: Props) {
	return (
		<StepProvider>
			<TaskList>
				<ProjectName args={args} />
				<Template />
				<PackageManager />
				<Directory />
				<SteelApiKey />
				<Dependencies />
			</TaskList>
		</StepProvider>
	);
}

// SCRATCHPAD
// Template template args defined in constants
// PackageManager
// SteelApiKey depending on base url
// Dependencies

// For --task and --base-url, these are often inside of the starter projects and aren't env variables.
// It might just be easiest to have these all be env variables and then update them in the starter projects.
