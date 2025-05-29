import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Delete File by ID';

export default function DeleteFileById() {
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
