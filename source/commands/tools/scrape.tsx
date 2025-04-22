import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function Scrape() {
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
									name: 'screenshot',
									type: 'boolean',
									initialValue: false,
									label: 'Screenshot',
									required: false,
								},
								{
									name: 'format',
									type: 'select',
									initialValue: 'html',
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
