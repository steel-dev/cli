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
	// zod
	// 	.string()
	// 	.default('steel-project')
	// 	.describe('Directory to scaffold new Steel project'),
	zod.string().describe('Example Project to start'),
]);

export const options = zod.object({
	name: zod.string().describe(
		option({
			description: 'Name of the project',
			alias: 'n',
		}),
	),
	'api-key': zod.string().describe('API Key for Steel API'),
	'skip-auth': zod.boolean().describe('Skip authentication'),
});

type Props = {
	args: zod.infer<typeof args>;
};

export default function Cookbook({args}: Props) {
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
