import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function getSessionsById() {
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
