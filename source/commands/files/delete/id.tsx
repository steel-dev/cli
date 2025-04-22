import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function deleteFileById() {
	return (
		<ApiDashboard
			method="DELETE"
			endpoint="sessions/{sessionId}/files/{fileId}"
			form={{
				form: {
					title: 'Delete File by ID',
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
