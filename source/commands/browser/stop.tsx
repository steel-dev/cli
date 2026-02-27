#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {
	stopBrowserSession,
	type StopBrowserSessionResult,
} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';

export const description = 'Stop the active Steel browser session';

export const options = zod.object({
	all: zod
		.boolean()
		.describe(
			option({
				description: 'Stop all live sessions in the active mode',
				alias: 'a',
			}),
		)
		.optional(),
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Stop sessions from local Steel runtime mode',
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

function printResult(result: StopBrowserSessionResult): void {
	if (result.stoppedSessionIds.length === 0) {
		console.log('No active browser sessions to stop.');
		return;
	}

	if (result.all) {
		console.log(
			`Stopped ${result.stoppedSessionIds.length} sessions in ${result.mode} mode.`,
		);
	} else {
		console.log(`Stopped session ${result.stoppedSessionIds[0]}.`);
	}
}

export default function Stop({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const result = await stopBrowserSession({
					all: options.all,
					local: options.local,
					apiUrl: options.apiUrl,
				});

				printResult(result);
				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to stop browser session. Check your network/auth and try again.',
					);
				}

				process.exit(1);
			}
		}

		run();
	}, [options.all, options.apiUrl, options.local]);
}
