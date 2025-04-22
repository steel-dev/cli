import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function Pdf() {
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
