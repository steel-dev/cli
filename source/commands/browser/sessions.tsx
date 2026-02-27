#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {listBrowserSessions} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';
import {formatBrowserSessionsForOutput} from '../../utils/browser/display.js';

export const description = 'List browser sessions as JSON';

export const options = zod.object({
	local: zod
		.boolean()
		.describe(
			option({
				description: 'List sessions from local Steel runtime',
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
	raw: zod
		.boolean()
		.describe(
			option({
				description: 'Include full raw API payload for each session',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Sessions({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const sessions = await listBrowserSessions({
					local: options.local,
					apiUrl: options.apiUrl,
				});
				const output = formatBrowserSessionsForOutput(sessions, {
					raw: options.raw,
				});

				console.log(JSON.stringify(output, null, 2));
				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to list browser sessions.');
				}

				process.exit(1);
			}
		}

		run();
	}, [options.apiUrl, options.local, options.raw]);
}
