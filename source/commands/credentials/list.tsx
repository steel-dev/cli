#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {listCredentials} from '../../utils/credentials-api.js';

export const description = 'List stored credentials';

export const options = zod.object({
	namespace: zod
		.string()
		.describe(
			option({
				description: 'Filter credentials by namespace',
				alias: 'n',
			}),
		)
		.optional(),
	origin: zod
		.string()
		.describe(
			option({
				description: 'Filter credentials by origin URL',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function List({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const credentials = await listCredentials({
					namespace: options.namespace?.trim() || undefined,
					origin: options.origin?.trim() || undefined,
				});

				console.log(JSON.stringify(credentials, null, 2));

				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to list credentials. Check your network/auth and try again.',
					);
				}

				process.exit(1);
			}
		}

		run();
	}, [options.namespace, options.origin]);
}
