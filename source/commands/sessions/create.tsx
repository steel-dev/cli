import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Create Session';

export default function CreateSession() {
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
								},
								{
									name: 'userAgent',
									type: 'string',
									label: 'User Agent',
									required: false,
								},
								{
									name: 'useProxy',
									type: 'boolean',
									label: 'Use Proxy',
									required: false,
								},
								{
									name: 'proxyUrl',
									type: 'string',
									label: 'Proxy URL',
									required: false,
								},
								{
									name: 'blockAds',
									type: 'boolean',
									label: 'Block Ads',
									required: false,
								},
								{
									name: 'solveCaptcha',
									type: 'boolean',
									label: 'Solve Captcha',
									required: false,
								},
								{
									name: 'timeout',
									type: 'integer',
									label: 'Timeout',
									required: false,
								},
								{
									name: 'concurrency',
									type: 'integer',
									label: 'Concurrency',
									required: false,
								},
								{
									name: 'isSelenium',
									type: 'boolean',
									label: 'Is Selenium',
									required: false,
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
								},
								{
									name: 'height',
									type: 'integer',
									label: 'Height',
									required: true,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
