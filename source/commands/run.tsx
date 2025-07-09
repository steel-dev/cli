import EnvVar from '../components/forge/envvar.js';
import Template from '../components/run/template.js';
import {TaskList} from 'ink-task-list';
import Dependencies from '../components/forge/dependencies.js';
import {StepProvider} from '../context/stepcontext.js';
import zod from 'zod';
import {option} from 'pastel';
// import PackageManager from '../components/forge/packagemanager.js';
import BrowserOpener from '../components/run/browseropener.js';

export const description = 'Start a new project using the Steel CLI';

export const args = zod.tuple([
	zod.string().describe('Example Project to run'),
]);

export const options = zod.object({
	'api-url': zod.string().describe(
		option({
			description: 'API URL for Steel API',
			alias: 'a',
		}),
	),
	view: zod.string().describe(
		option({
			description: 'Auto open live session viewer',
			alias: 'v',
		}),
	),
	task: zod.string().describe(
		option({
			description: 'Task to run',
			alias: 't',
		}),
	),
	'api-key': zod.string().describe('API Key for Steel API'),
	'openai-key': zod.string().describe('API Key for OpenAI'),
	'skip-auth': zod.boolean().describe('Skip authentication'),
});

type Props = {
	args: zod.infer<typeof args>;
	options: zod.infer<typeof options>;
};

export default function Cookbook({args, options}: Props) {
	return (
		<StepProvider>
			<TaskList>
				<Template args={args} />
				{/* <PackageManager /> */}
				<EnvVar options={options} />
				<Dependencies />
				<BrowserOpener options={options} />
			</TaskList>
		</StepProvider>
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
