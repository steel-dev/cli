#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {createCredential} from '../../utils/credentials-api.js';

export const description = 'Store a new credential for a given origin';

export const options = zod.object({
	origin: zod
		.string()
		.describe(
			option({
				description:
					'Origin URL to associate the credential with (e.g. https://example.com)',
			}),
		)
		.optional(),
	username: zod
		.string()
		.describe(
			option({
				description: 'Username for the credential',
				alias: 'u',
			}),
		)
		.optional(),
	password: zod
		.string()
		.describe(
			option({
				description: 'Password for the credential',
				alias: 'p',
			}),
		)
		.optional(),
	totpSecret: zod
		.string()
		.describe(
			option({
				description: 'TOTP secret for two-factor authentication (optional)',
			}),
		)
		.optional(),
	namespace: zod
		.string()
		.describe(
			option({
				description: 'Credential namespace (optional)',
				alias: 'n',
			}),
		)
		.optional(),
	label: zod
		.string()
		.describe(
			option({
				description: 'Human-readable label for the credential (optional)',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Create({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const origin = options.origin?.trim();
				const username = options.username?.trim();
				const password = options.password?.trim();
				const totpSecret = options.totpSecret?.trim() || undefined;
				const namespace = options.namespace?.trim() || undefined;
				const label = options.label?.trim() || undefined;

				if (!origin) {
					console.error('Missing required flag: --origin');
					process.exit(1);
					return;
				}

				if (!username) {
					console.error('Missing required flag: --username');
					process.exit(1);
					return;
				}

				if (!password) {
					console.error('Missing required flag: --password');
					process.exit(1);
					return;
				}

				const result = await createCredential({
					origin,
					value: {username, password, totpSecret},
					namespace,
					label,
				});

				console.log(`origin: ${origin}`);
				console.log(`username: ${username}`);
				if (namespace) {
					console.log(`namespace: ${namespace}`);
				}

				if (label) {
					console.log(`label: ${label}`);
				}

				if (result['id']) {
					console.log(`id: ${result['id']}`);
				}

				console.log('Credential created successfully.');

				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to create credential. Check your network/auth and try again.',
					);
				}

				process.exit(1);
			}
		}

		run();
	}, [
		options.origin,
		options.username,
		options.password,
		options.totpSecret,
		options.namespace,
		options.label,
	]);
}
