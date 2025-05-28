import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Download File';

export default function DownloadFile() {
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
