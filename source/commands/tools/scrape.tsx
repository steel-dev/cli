import React from 'react';
import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../components/apidashboard.js';

export const options = zod.object({
	url: zod
		.string()
		.url()
		.optional()
		.describe(
			option({
				description: 'The URL of the page to scrape',
			}),
		),
	format: zod
		.enum(['html', 'readability', 'cleaned_html', 'markdown'])
		.optional()
		.describe(option({description: 'Desired format of the scraped content'})),
	useProxy: zod
		.boolean()
		.optional()
		.describe(option({description: 'Use a proxy for the request'})),
	delay: zod
		.number()
		.min(0)
		.optional()
		.describe(option({description: 'Delay before scraping'})),
	screenshot: zod
		.boolean()
		.optional()
		.describe(option({description: 'Take a screenshot of the page'})),
	pdf: zod
		.boolean()
		.optional()
		.describe(option({description: 'Generate a PDF of the page'})),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Scrape({options}: Props) {
	return (
		<ApiDashboard
			method="POST"
			endpoint="scrape"
			form={{
				form: {
					title: 'Scrape',
					sections: [
						{
							title: 'Body',
							fields: [
								{
									name: 'url',
									type: 'string',
									label: 'URL',
									initialValue: options.url,
									required: true,
								},
								{
									name: 'useProxy',
									type: 'boolean',
									initialValue: options.useProxy,
									label: 'Use Proxy',
									required: false,
								},
								{
									name: 'delay',
									type: 'number',
									initialValue: options.delay,
									label: 'Delay',
									required: false,
								},
								{
									name: 'screenshot',
									type: 'boolean',
									initialValue: options.screenshot,
									label: 'Screenshot',
									required: false,
								},
								{
									name: 'pdf',
									type: 'boolean',
									initialValue: options.pdf,
									label: 'PDF',
									required: false,
								},
								{
									name: 'format',
									type: 'select',
									initialValue: options.format,
									options: [
										{label: 'HTML', value: 'html'},
										{label: 'Readability', value: 'readability'},
										{label: 'Cleaned HTML', value: 'cleaned_html'},
										{label: 'Markdown', value: 'markdown'},
									],
									label: 'Format',
									required: false,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
