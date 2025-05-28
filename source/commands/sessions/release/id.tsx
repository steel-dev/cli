import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Release Session By Id';

export default function ReleaseSessionById() {
	return (
		<ApiDashboard
			method="POST"
			endpoint="sessions/{id}/release"
			form={{
				form: {
					title: 'Release Session By Id',
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
