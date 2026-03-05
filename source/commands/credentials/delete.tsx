#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {deleteCredential} from '../../utils/credentials-api.js';

export const description = 'Delete a stored credential';

export const options = zod.object({
	origin: zod
		.string()
		.describe(
			option({
				description: 'Origin URL of the credential to delete',
			}),
		)
		.optional(),
	namespace: zod
		.string()
		.describe(
			option({
				description: 'Credential namespace',
				alias: 'n',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Delete({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const origin = options.origin?.trim();
				const namespace = options.namespace?.trim() || undefined;

				if (!origin) {
					console.error('Missing required flag: --origin');
					process.exit(1);
					return;
				}

				await deleteCredential({origin, namespace});

				console.log(`origin: ${origin}`);
				if (namespace) {
					console.log(`namespace: ${namespace}`);
				}

				console.log('Credential deleted successfully.');

				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to delete credential. Check your network/auth and try again.',
					);
				}

				process.exit(1);
			}
		}

		run();
	}, [options.origin, options.namespace]);
}
