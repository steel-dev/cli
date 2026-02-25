#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {getActiveBrowserLiveUrl} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';

export const description = 'Print active session live-view URL';

export const options = zod.object({
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Resolve live URL from local active session',
				alias: 'l',
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
					local: options.local,
				});

				if (!liveUrl) {
					console.error(
						'No active live session found. Start one with `steel browser start`.',
					);
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
	}, [options.local]);
}
