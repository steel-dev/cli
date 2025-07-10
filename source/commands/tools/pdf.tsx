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
	useProxy: zod
		.boolean()
		.optional()
		.describe(
			option({
				description: 'Whether to use a Steel-provided proxy',
			}),
		),
	delay: zod
		.number()
		.min(0)
		.optional()
		.describe(
			option({
				description: 'Delay in milliseconds',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Pdf({options}: Props) {
	return (
		<ApiDashboard
			method="POST"
			endpoint="pdf"
			form={{
				form: {
					title: 'Pdf',
					sections: [
						{
							title: 'Body',
							fields: [
								{
									name: 'url',
									type: 'string',
									label: 'URL',
									required: true,
									initialValue: options.url,
								},
								{
									name: 'useProxy',
									type: 'boolean',
									label: 'Use Proxy',
									required: false,
									initialValue: options.useProxy,
								},
								{
									name: 'delay',
									type: 'number',
									label: 'Delay',
									required: false,
									initialValue: options.delay,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
