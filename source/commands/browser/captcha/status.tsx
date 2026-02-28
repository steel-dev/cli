#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {getBrowserSessionCaptchaStatus} from '../../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../../utils/browser/errors.js';

export const description =
	'Get CAPTCHA solving status for a Steel browser session';

export const options = zod.object({
	sessionId: zod
		.string()
		.describe(
			option({
				description: 'Explicit Steel session id to target',
			}),
		)
		.optional(),
	session: zod
		.string()
		.describe(
			option({
				description: 'Named session key to resolve from local state',
				alias: 's',
			}),
		)
		.optional(),
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Resolve session and execute status call in local mode',
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
	pageId: zod
		.string()
		.describe(
			option({
				description: 'Optional page ID for targeted CAPTCHA status',
			}),
		)
		.optional(),
	wait: zod
		.boolean()
		.describe(
			option({
				description: 'Poll until CAPTCHA is resolved (solved/failed/none)',
				alias: 'w',
			}),
		)
		.optional(),
	timeout: zod.coerce
		.number()
		.int()
		.positive()
		.describe(
			option({
				description: 'Timeout in milliseconds for --wait mode (default: 60000)',
			}),
		)
		.optional(),
	interval: zod.coerce
		.number()
		.int()
		.positive()
		.describe(
			option({
				description:
					'Poll interval in milliseconds for --wait mode (default: 1000)',
			}),
		)
		.optional(),
	raw: zod
		.boolean()
		.describe(
			option({
				description: 'Print the full raw API payload (JSON)',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function CaptchaStatus({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const result = await getBrowserSessionCaptchaStatus({
					sessionId: options.sessionId,
					sessionName: options.session,
					local: options.local,
					apiUrl: options.apiUrl,
					pageId: options.pageId,
					wait: options.wait,
					timeout: options.timeout,
					interval: options.interval,
				});

				if (options.raw) {
					console.log(JSON.stringify(result.raw, null, 2));
				} else {
					const typeSuffix =
						result.types.length > 0 ? ` ${result.types.join(',')}` : '';
					console.log(`${result.status}${typeSuffix}`);
				}

				const exitCode =
					result.status === 'solved' || result.status === 'none' ? 0 : 1;
				process.exit(exitCode);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to get CAPTCHA status.');
				}

				process.exit(1);
			}
		}

		run();
	}, [
		options.apiUrl,
		options.interval,
		options.local,
		options.pageId,
		options.raw,
		options.sessionId,
		options.session,
		options.timeout,
		options.wait,
	]);
}
