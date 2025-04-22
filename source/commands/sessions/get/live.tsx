import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function getSessionLiveDetailsById() {
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
