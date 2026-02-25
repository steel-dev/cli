#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {startBrowserSession} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';

export const description = 'Create or attach a Steel browser session';

export const options = zod.object({
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Start or attach a local Steel browser session',
				alias: 'l',
			}),
		)
		.optional(),
	session: zod
		.string()
		.describe(
			option({
				description: 'Named session key for create-or-attach behavior',
				alias: 's',
			}),
		)
		.optional(),
	stealth: zod
		.boolean()
		.describe(
			option({
				description: 'Enable stealth-oriented session defaults',
			}),
		)
		.optional(),
	proxy: zod
		.string()
		.describe(
			option({
				description: 'Proxy URL to apply when creating a new session',
				alias: 'p',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Start({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const session = await startBrowserSession({
					local: options.local,
					sessionName: options.session,
					stealth: options.stealth,
					proxyUrl: options.proxy,
				});

				console.log(`id: ${session.id}`);
				console.log(`mode: ${session.mode}`);
				if (session.name) {
					console.log(`name: ${session.name}`);
				}

				if (session.viewerUrl) {
					console.log(`live_url: ${session.viewerUrl}`);
				}

				if (session.connectUrl) {
					console.log(`connect_url: ${session.connectUrl}`);
				}

				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error(
						'Failed to start browser session. Check your network/auth and try again.',
					);
				}

				process.exit(1);
			}
		}

		run();
	}, [options.local, options.proxy, options.session, options.stealth]);
}
