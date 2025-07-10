import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Create Session';

export const options = zod.object({
	sessionId: zod
		.string()
		.uuid()
		.optional()
		.describe(
			option({
				description: 'The ID of the session',
			}),
		),
	userAgent: zod
		.string()
		.min(1)
		.optional()
		.describe(
			option({
				description: 'The User Agent of the session',
			}),
		),
	useProxy: zod
		.boolean()
		.optional()
		.describe(
			option({
				description: 'Whether to use a proxy for the session',
			}),
		),
	proxyUrl: zod
		.string()
		.url()
		.optional()
		.describe(
			option({
				description: 'The URL of the proxy to use for the session',
			}),
		),
	blockAds: zod
		.boolean()
		.optional()
		.describe(
			option({
				description: 'Whether to block ads for the session',
			}),
		),
	solveCaptcha: zod
		.boolean()
		.optional()
		.describe(
			option({
				description: 'Whether to solve captchas for the session',
			}),
		),
	timeout: zod
		.number()
		.min(1)
		.optional()
		.describe(
			option({
				description: 'The timeout for the session',
			}),
		),
	concurrency: zod
		.number()
		.min(1)
		.optional()
		.describe(
			option({
				description: 'The number of concurrent sessions to create',
			}),
		),
	isSelenium: zod
		.boolean()
		.optional()
		.describe(
			option({
				description: 'Whether to use Selenium for the session',
			}),
		),
	width: zod
		.number()
		.min(1)
		.optional()
		.describe(
			option({
				description: 'The width for the browser in the session',
			}),
		),
	height: zod
		.number()
		.min(1)
		.optional()
		.describe(
			option({
				description: 'The height for the browser in the session',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function CreateSession({options}: Props) {
	return (
		<ApiDashboard
			method="POST"
			endpoint="sessions"
			form={{
				form: {
					title: 'Create Session',
					sections: [
						{
							title: 'Session Details',
							fields: [
								{
									name: 'sessionId',
									type: 'string',
									label: 'Session ID',
									required: false,
									initialValue: options.sessionId,
								},
								{
									name: 'userAgent',
									type: 'string',
									label: 'User Agent',
									required: false,
									initialValue: options.userAgent,
								},
								{
									name: 'useProxy',
									type: 'boolean',
									label: 'Use Proxy',
									required: false,
									initialValue: options.useProxy,
								},
								{
									name: 'proxyUrl',
									type: 'string',
									label: 'Proxy URL',
									required: false,
									initialValue: options.proxyUrl,
								},
								{
									name: 'blockAds',
									type: 'boolean',
									label: 'Block Ads',
									required: false,
									initialValue: options.blockAds,
								},
								{
									name: 'solveCaptcha',
									type: 'boolean',
									label: 'Solve Captcha',
									required: false,
									initialValue: options.solveCaptcha,
								},
								{
									name: 'timeout',
									type: 'integer',
									label: 'Timeout',
									required: false,
									initialValue: options.timeout,
								},
								{
									name: 'concurrency',
									type: 'integer',
									label: 'Concurrency',
									required: false,
									initialValue: options.concurrency,
								},
								{
									name: 'isSelenium',
									type: 'boolean',
									label: 'Is Selenium',
									required: false,
									initialValue: options.isSelenium,
								},
							],
						},
						{
							title: 'Dimensions',
							fields: [
								{
									name: 'width',
									type: 'integer',
									label: 'Width',
									required: true,
									initialValue: options.width,
								},
								{
									name: 'height',
									type: 'integer',
									label: 'Height',
									required: true,
									initialValue: options.height,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
