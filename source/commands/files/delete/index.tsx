import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function deleteFiles() {
	return (
		<ApiDashboard
			method="DELETE"
			endpoint="sessions/{sessionId}/files"
			resultObject="data"
			form={{
				form: {
					title: 'Delete Files By Session',
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
