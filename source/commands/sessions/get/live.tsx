import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Get Session Live Details By Id';

export default function SessionLiveDetailsById() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}/live-details"
			form={{
				form: {
					title: 'Get Session Live Details By Id',
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
