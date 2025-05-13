import React from 'react';
import SteelApiKey from '../components/cookbook/steelapikey.js';
import Template from '../components/cookbook/template.js';
import Directory from '../components/cookbook/directory.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/cookbook/dependencies.js';
import ProjectName from '../components/cookbook/projectname.js';
import {StepProvider} from '../context/stepcontext.js';
import zod from 'zod';
import PackageManager from '../components/cookbook/packagemanager.js';

export const args = zod.tuple([
	zod
		.string()
		.default('steel-project')
		.describe('Directory to scaffold new Steel project'),
]);

type Props = {
	args: zod.infer<typeof args>;
};

export default function Cookbook({args}: Props) {
	return (
		<StepProvider>
			<TaskList>
				<Template />
				<PackageManager />
				<Directory args={args} />
				<ProjectName args={args} />
				<SteelApiKey />
				<Dependencies />
			</TaskList>
		</StepProvider>
	);
}
