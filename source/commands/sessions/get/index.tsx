import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Get Session By Id';

export default function SessionsById() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}"
			form={{
				form: {
					title: 'Get Session By Id',
					sections: [
						{
							title: 'Session Details',
							fields: [
								{
									name: 'id',
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
