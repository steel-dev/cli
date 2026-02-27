#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {BrowserAdapterError} from '../utils/browser/errors.js';
import {
	getHostedAssetUrl,
	requestTopLevelApi,
	resolveTopLevelToolUrl,
} from '../utils/topLevelTools.js';

export const description = 'Capture a webpage screenshot through the Steel API';

export const args = zod.tuple([
	zod.string().describe('Target URL to capture').optional(),
]);

export const argsLabels = ['url'];

export const options = zod.object({
	url: zod
		.string()
		.describe(
			option({
				description: 'Target URL to capture',
				alias: 'u',
			}),
		)
		.optional(),
	delay: zod
		.number()
		.describe(
			option({
				description: 'Delay before capture in milliseconds',
				alias: 'd',
			}),
		)
		.optional(),
	fullPage: zod
		.boolean()
		.describe(
			option({
				description: 'Capture the full page (not only the viewport)',
				alias: 'f',
			}),
		)
		.optional(),
	useProxy: zod
		.boolean()
		.describe(
			option({
				description: 'Use a Steel-managed residential proxy',
			}),
		)
		.optional(),
	region: zod
		.string()
		.describe(
			option({
				description: 'Region identifier for request execution',
				alias: 'r',
			}),
		)
		.optional(),
	local: zod
		.boolean()
		.describe(
			option({
				description: 'Send request to local Steel runtime mode',
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
	args: zod.infer<typeof args>;
	options: zod.infer<typeof options>;
};

export default function Screenshot({args, options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const url = resolveTopLevelToolUrl(options.url, args[0]);

				const response = await requestTopLevelApi(
					'/screenshot',
					{
						url,
						delay: options.delay,
						fullPage: options.fullPage,
						useProxy: options.useProxy,
						region: options.region,
					},
					{
						local: options.local,
						apiUrl: options.apiUrl,
					},
				);

				console.log(getHostedAssetUrl(response));
				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to capture screenshot.');
				}

				process.exit(1);
			}
		}

		run();
	}, [
		args,
		options.apiUrl,
		options.delay,
		options.fullPage,
		options.local,
		options.region,
		options.url,
		options.useProxy,
	]);
}
