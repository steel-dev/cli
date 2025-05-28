import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Get Session Events By Id';

export default function SessionEventsById() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}/events"
			form={{
				form: {
					title: 'Get Session Events By Id',
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
