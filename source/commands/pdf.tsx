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

export const description = 'Generate a webpage PDF through the Steel API';

export const args = zod.tuple([
	zod.string().describe('Target URL to convert').optional(),
]);

export const argsLabels = ['url'];

export const options = zod.object({
	url: zod
		.string()
		.describe(
			option({
				description: 'Target URL to convert',
				alias: 'u',
			}),
		)
		.optional(),
	delay: zod
		.number()
		.describe(
			option({
				description: 'Delay before PDF generation in milliseconds',
				alias: 'd',
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

export default function Pdf({args, options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const url = resolveTopLevelToolUrl(options.url, args[0]);

				const response = await requestTopLevelApi(
					'/pdf',
					{
						url,
						delay: options.delay,
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
					console.error('Failed to generate PDF.');
				}

				process.exit(1);
			}
		}

		run();
	}, [
		args,
		options.apiUrl,
		options.delay,
		options.local,
		options.region,
		options.url,
		options.useProxy,
	]);
}
