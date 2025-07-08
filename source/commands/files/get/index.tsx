import React from 'react';
import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Get File By ID';

export const options = zod.object({
	sessionId: zod
		.string()
		.uuid()
		.min(1)
		.max(36)
		.optional()
		.describe(
			option({
				description: 'The ID of the session',
			}),
		),
	fileId: zod
		.string()
		.uuid()
		.min(1)
		.max(36)
		.optional()
		.describe(
			option({
				description: 'The ID of the file to delete',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function FileById({options}: Props) {
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
									initialValue: options.sessionId,
								},
								{
									name: 'fileId',
									type: 'string',
									label: 'File ID',
									required: true,
									initialValue: options.fileId,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
