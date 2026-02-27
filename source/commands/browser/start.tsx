#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {startBrowserSession} from '../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../utils/browser/errors.js';
import {sanitizeConnectUrlForDisplay} from '../../utils/browser/display.js';

export const description =
	'Create or attach a Steel browser session (cloud by default)';

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
	apiUrl: zod
		.string()
		.describe(
			option({
				description: 'Explicit self-hosted API endpoint URL',
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
				description:
					'Apply stealth preset on new sessions (humanized interactions + auto CAPTCHA solving)',
			}),
		)
		.optional(),
	proxy: zod
		.string()
		.describe(
			option({
				description:
					'Proxy URL for new sessions (for example, http://user:pass@host:port)',
				alias: 'p',
			}),
		)
		.optional(),
	sessionTimeout: zod.coerce
		.number()
		.int()
		.positive()
		.describe(
			option({
				description: 'Session timeout in milliseconds (create-time only)',
			}),
		)
		.optional(),
	sessionHeadless: zod
		.boolean()
		.describe(
			option({
				description: 'Create new sessions in headless mode (create-time only)',
			}),
		)
		.optional(),
	sessionRegion: zod
		.string()
		.describe(
			option({
				description: 'Preferred session region (create-time only)',
			}),
		)
		.optional(),
	sessionSolveCaptcha: zod
		.boolean()
		.describe(
			option({
				description:
					'Enable CAPTCHA solving on new sessions (create-time only)',
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
					apiUrl: options.apiUrl,
					sessionName: options.session,
					stealth: options.stealth,
					proxyUrl: options.proxy,
					timeoutMs: options.sessionTimeout,
					headless: options.sessionHeadless,
					region: options.sessionRegion,
					solveCaptcha: options.sessionSolveCaptcha,
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
					console.log(
						`connect_url: ${sanitizeConnectUrlForDisplay(session.connectUrl)}`,
					);
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
	}, [
		options.apiUrl,
		options.local,
		options.proxy,
		options.sessionHeadless,
		options.sessionRegion,
		options.sessionSolveCaptcha,
		options.sessionTimeout,
		options.session,
		options.stealth,
	]);
}
