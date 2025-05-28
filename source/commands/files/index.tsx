import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Get Files By Session';

export default function Files() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{sessionId}/files"
			resultObject="data"
			form={{
				form: {
					title: 'Get Files By Session',
					sections: [
						{
							title: 'Session Details',
							fields: [
								{
									name: 'sessionId',
									type: 'string',
									label: 'Session ID',
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
