import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function getFiles() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{sessionId}/files/{fileId}"
			form={{
				form: {
					title: 'Get File by ID',
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
								{
									name: 'fileId',
									type: 'string',
									label: 'File ID',
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
