#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {getActiveBrowserLiveUrl} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';

export const description = 'Print active or named session live-view URL';

export const options = zod.object({
	session: zod
		.string()
		.describe(
			option({
				description: 'Named session key to resolve live URL for',
				alias: 's',
			}),
		)
		.optional(),
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Resolve live URL from local active session',
				alias: 'l',
			}),
		)
		.optional(),
	apiUrl: zod
		.string()
		.describe(
			option({
				description: 'Explicit self-hosted API endpoint URL',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Live({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const liveUrl = await getActiveBrowserLiveUrl({
					sessionName: options.session,
					local: options.local,
					apiUrl: options.apiUrl,
				});

				if (!liveUrl) {
					const sessionName = options.session?.trim();
					if (sessionName) {
						console.error(
							`No live session found for "${sessionName}". Start one with \`steel browser start --session ${sessionName}\`.`,
						);
					} else {
						console.error(
							'No active live session found. Start one with `steel browser start`.',
						);
					}

					process.exit(1);
					return;
				}

				console.log(liveUrl);
				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to resolve browser live URL.');
				}

				process.exit(1);
			}
		}

		run();
	}, [options.apiUrl, options.local, options.session]);
}
