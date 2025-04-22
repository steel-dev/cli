import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function Screenshot() {
	return (
		<ApiDashboard
			method="POST"
			endpoint="screenshot"
			form={{
				form: {
					title: 'Screenshot',
					sections: [
						{
							title: 'Body',
							fields: [
								{
									name: 'url',
									type: 'string',
									label: 'URL',
									required: true,
								},
								{
									name: 'useProxy',
									type: 'boolean',
									initialValue: false,
									label: 'Use Proxy',
									required: false,
								},
								{
									name: 'delay',
									type: 'number',
									initialValue: 1,
									label: 'Delay',
									required: false,
								},
								{
									name: 'fullPage',
									type: 'boolean',
									initialValue: false,
									label: 'Full Page',
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
