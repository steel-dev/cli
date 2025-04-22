import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function downloadFile() {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{sessionId}/files/{fileId}/download"
			form={{
				form: {
					title: 'Download File',
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
