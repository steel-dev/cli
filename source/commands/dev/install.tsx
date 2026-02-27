#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {installLocalBrowserRuntime} from '../../utils/dev/local.js';

export const description =
	'Install local Steel Browser runtime assets without starting containers';

export const options = zod.object({
	repoUrl: zod
		.string()
		.describe(
			option({
				description: 'Git repository URL for local Steel Browser runtime',
			}),
		)
		.optional(),
	verbose: zod
		.boolean()
		.describe(
			option({
				description: 'Enable verbose git command output',
				alias: 'V',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function DevInstall({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const result = installLocalBrowserRuntime({
					repoUrl: options.repoUrl,
					verbose: options.verbose,
				});

				if (result.installed) {
					console.log('Local Steel Browser runtime installed.');
				} else {
					console.log('Local Steel Browser runtime already installed.');
				}

				console.log(`repo_path: ${result.repoPath}`);
				console.log(`repo_url: ${result.repoUrl}`);
				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to install local Steel Browser runtime.');
				}

				process.exit(1);
			}
		}

		run();
	}, [options.repoUrl, options.verbose]);

	return null;
}
