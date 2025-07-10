import Template from '../components/run/template.js';
import Runner from '../components/run/runner.js';
import EnvVar from '../components/run/envvar.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/run/dependencies.js';
import {RunStepProvider} from '../context/runstepcontext.js';
import zod from 'zod';
import {option} from 'pastel';
import BrowserOpener from '../components/run/browseropener.js';
import Directory from '../components/run/directory.js';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example Project to run').optional(),
]);

export const options = zod.object({
	'api-url': zod
		.string()
		.describe(
			option({
				description: 'API URL for Steel API',
				alias: 'a',
			}),
		)
		.optional(),
	view: zod
		.string()
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
	'api-key': zod.string().describe('API Key for Steel API').optional(),
	'openai-key': zod.string().describe('API Key for OpenAI').optional(),
	'skip-auth': zod.boolean().describe('Skip authentication').optional(),
});

type Props = {
	args: zod.infer<typeof args>;
	options: zod.infer<typeof options>;
};

export default function Cookbook({args, options}: Props) {
	return (
		<RunStepProvider>
			<TaskList>
				<Template args={args} />
				<Directory />
				<EnvVar options={options} />
				<Dependencies />
				<Runner />
				{options['view'] && <BrowserOpener options={options} />}
			</TaskList>
		</RunStepProvider>
	);
}

// SCRATCHPAD
// Template template args defined in constants
// PackageManager (don't know if this is needed), but i'm debating how to handle installing packages
// I don't think this is needed, but then how do we handle installing stuff? just use the default commands or whatever.
// EnvVar to update the Env Variables
// Dependencies (Ask to install these)

// For --task and --base-url, these are often inside of the starter projects and aren't env variables.
// It might just be easiest to have these all be env variables and then update them in the starter projects.
