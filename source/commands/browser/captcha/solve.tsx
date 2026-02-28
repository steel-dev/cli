#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {solveBrowserSessionCaptcha} from '../../../utils/browser/lifecycle.js';
import {BrowserAdapterError} from '../../../utils/browser/errors.js';

export const description =
	'Manually trigger CAPTCHA solving for a Steel browser session';

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
				description: 'Resolve session and execute solve call in local mode',
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
				description: 'Optional page ID for targeted CAPTCHA solving',
			}),
		)
		.optional(),
	url: zod
		.string()
		.describe(
			option({
				description: 'Optional page URL for targeted CAPTCHA solving',
			}),
		)
		.optional(),
	taskId: zod
		.string()
		.describe(
			option({
				description: 'Optional CAPTCHA task ID for targeted solving',
			}),
		)
		.optional(),
	raw: zod
		.boolean()
		.describe(
			option({
				description: 'Print the full raw API payload',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function SolveCaptcha({options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const result = await solveBrowserSessionCaptcha({
					sessionId: options.sessionId,
					sessionName: options.session,
					local: options.local,
					apiUrl: options.apiUrl,
					pageId: options.pageId,
					url: options.url,
					taskId: options.taskId,
				});

				console.log(`session_id: ${result.sessionId}`);
				console.log(`mode: ${result.mode}`);
				console.log(`success: ${result.success}`);
				if (result.message) {
					console.log(`message: ${result.message}`);
				}

				if (options.raw) {
					console.log(JSON.stringify(result.raw, null, 2));
				}

				process.exit(result.success ? 0 : 1);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to trigger CAPTCHA solving.');
				}

				process.exit(1);
			}
		}

		run();
	}, [
		options.apiUrl,
		options.local,
		options.pageId,
		options.raw,
		options.sessionId,
		options.session,
		options.taskId,
		options.url,
	]);
}
