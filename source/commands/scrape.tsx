#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {BrowserAdapterError} from '../utils/browser/errors.js';
import {
	getScrapeOutputText,
	parseScrapeFormatOption,
	requestTopLevelApi,
	resolveTopLevelToolUrl,
} from '../utils/topLevelTools.js';

export const description =
	'Scrape webpage content through the Steel API (markdown output by default)';

export const args = zod.tuple([
	zod.string().describe('Target URL to scrape').optional(),
]);

export const argsLabels = ['url'];

export const options = zod.object({
	url: zod
		.string()
		.describe(
			option({
				description: 'Target URL to scrape',
				alias: 'u',
			}),
		)
		.optional(),
	format: zod
		.string()
		.describe(
			option({
				description:
					'Comma-separated output formats: html, readability, cleaned_html, markdown',
			}),
		)
		.optional(),
	raw: zod
		.boolean()
		.describe(
			option({
				description: 'Print full JSON response payload',
			}),
		)
		.optional(),
	delay: zod
		.number()
		.describe(
			option({
				description: 'Delay before scraping in milliseconds',
				alias: 'd',
			}),
		)
		.optional(),
	pdf: zod
		.boolean()
		.describe(
			option({
				description: 'Include a generated PDF in the scrape response',
			}),
		)
		.optional(),
	screenshot: zod
		.boolean()
		.describe(
			option({
				description: 'Include a generated screenshot in the scrape response',
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

export default function Scrape({args, options}: Props) {
	useEffect(() => {
		async function run() {
			try {
				const url = resolveTopLevelToolUrl(options.url, args[0]);
				const format = parseScrapeFormatOption(options.format) || ['markdown'];

				const response = await requestTopLevelApi(
					'/scrape',
					{
						url,
						delay: options.delay,
						format,
						pdf: options.pdf,
						screenshot: options.screenshot,
						useProxy: options.useProxy,
						region: options.region,
					},
					{
						local: options.local,
						apiUrl: options.apiUrl,
					},
				);

				if (options.raw) {
					console.log(JSON.stringify(response, null, 2));
					process.exit(0);
					return;
				}

				const scrapeOutput = getScrapeOutputText(response, format);
				if (scrapeOutput) {
					console.log(scrapeOutput);
				} else {
					console.log(JSON.stringify(response, null, 2));
				}

				process.exit(0);
			} catch (error) {
				if (error instanceof BrowserAdapterError) {
					console.error(error.message);
				} else if (error instanceof Error) {
					console.error(error.message);
				} else {
					console.error('Failed to scrape URL.');
				}

				process.exit(1);
			}
		}

		run();
	}, [
		args,
		options.apiUrl,
		options.delay,
		options.format,
		options.local,
		options.pdf,
		options.raw,
		options.region,
		options.screenshot,
		options.url,
		options.useProxy,
	]);
}
