import React from 'react';
import PostDashboard from '../../components/postdashboard.js';

export default function PDF() {
	return (
		<PostDashboard
			form={{
				form: {
					title: 'PDF',
					sections: [
						{
							title: 'url',
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
