import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function getSessionContextById() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}/context"
			form={{
				form: {
					title: 'Get Session Context By Id',
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
