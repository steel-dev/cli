#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {
	isDockerRunning,
	startLocalBrowserRuntime,
} from '../../utils/dev/local.js';

export const description =
	'Start local Steel Browser runtime containers (requires `steel dev install`)';

export const options = zod.object({
	port: zod
		.number()
		.describe(
			option({
				description: 'API port for local Steel Browser runtime',
				alias: 'p',
			}),
		)
		.optional(),
	verbose: zod
		.boolean()
		.describe(
			option({
				description: 'Enable verbose Docker command output',
				alias: 'V',
			}),
		)
		.optional(),
	docker_check: zod
		.boolean()
		.describe(
			option({
				description: 'Only verify Docker availability and exit',
				alias: 'd',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function DevStart({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				if (options.docker_check) {
					if (isDockerRunning()) {
						console.log('Docker is running.');
						process.exit(0);
						return;
					}

					console.error('Docker is not running.');
					process.exit(1);
					return;
				}

				const result = startLocalBrowserRuntime({
					port: options.port,
					verbose: options.verbose,
				});

				console.log('Local Steel Browser runtime started.');
				console.log(`repo_path: ${result.repoPath}`);
				console.log(`api_port: ${result.apiPort}`);
				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to start local Steel Browser runtime.');
				}

				process.exit(1);
			}
		}

		run();
	}, [options.docker_check, options.port, options.verbose]);

	return null;
}
