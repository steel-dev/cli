import EnvVar from '../components/forge/envvar.js';
import Template from '../components/forge/template.js';
import Directory from '../components/forge/directory.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/forge/dependencies.js';
import ProjectName from '../components/forge/projectname.js';
import {ForgeStepProvider} from '../context/forgestepcontext.js';
import zod from 'zod';
import {option} from 'pastel';
import PackageManager from '../components/forge/packagemanager.js';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example Project to start').optional(),
]);

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
	'api-key': zod.string().describe('API Key for Steel API').optional(),
	'openai-key': zod.string().describe('API Key for OpenAI').optional(),
	'skip-auth': zod.boolean().describe('Skip authentication').optional(),
});

type Props = {
	args: zod.infer<typeof args>;
	options: zod.infer<typeof options>;
};

export default function Forge({args, options}: Props) {
	return (
		<ForgeStepProvider>
			<TaskList>
				<ProjectName args={args} />
				<Template args={args} />
				<PackageManager />
				<Directory />
				<EnvVar options={options} />
				<Dependencies />
			</TaskList>
		</ForgeStepProvider>
	);
}
