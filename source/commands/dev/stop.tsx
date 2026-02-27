#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {stopLocalBrowserRuntime} from '../../utils/dev/local.js';

export const description = 'Stop local Steel Browser runtime containers';

export const options = zod.object({
	verbose: zod
		.boolean()
		.describe(
			option({
				description: 'Enable verbose Docker command output',
				alias: 'v',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function DevStop({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const result = stopLocalBrowserRuntime({
					verbose: options.verbose,
				});

				console.log('Local Steel Browser runtime stopped.');
				console.log(`repo_path: ${result.repoPath}`);
				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to stop local Steel Browser runtime.');
				}

				process.exit(1);
			}
		}

		run();
	}, [options.verbose]);

	return null;
}
