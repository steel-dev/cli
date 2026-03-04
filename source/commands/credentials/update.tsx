#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {updateCredential} from '../../utils/credentials-api.js';

export const description = 'Update an existing credential';

export const options = zod.object({
	origin: zod
		.string()
		.describe(
			option({
				description: 'Origin URL of the credential to update',
			}),
		)
		.optional(),
	username: zod
		.string()
		.describe(
			option({
				description: 'New username',
				alias: 'u',
			}),
		)
		.optional(),
	password: zod
		.string()
		.describe(
			option({
				description: 'New password',
				alias: 'p',
			}),
		)
		.optional(),
	totpSecret: zod
		.string()
		.describe(
			option({
				description: 'New TOTP secret for two-factor authentication',
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
	label: zod
		.string()
		.describe(
			option({
				description: 'New human-readable label',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Update({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const origin = options.origin?.trim();
				const username = options.username?.trim() || undefined;
				const password = options.password?.trim() || undefined;
				const totpSecret = options.totpSecret?.trim() || undefined;
				const namespace = options.namespace?.trim() || undefined;
				const label = options.label?.trim() || undefined;

				if (!origin) {
					console.error('Missing required flag: --origin');
					process.exit(1);
					return;
				}

				const result = await updateCredential({
					origin,
					value:
						username || password || totpSecret
							? {username, password, totpSecret}
							: undefined,
					namespace,
					label,
				});

				console.log(`origin: ${origin}`);
				if (namespace) {
					console.log(`namespace: ${namespace}`);
				}

				if (result['id']) {
					console.log(`id: ${result['id']}`);
				}

				console.log('Credential updated successfully.');

				process.exit(0);
			} catch (error) {
				if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to update credential. Check your network/auth and try again.',
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
