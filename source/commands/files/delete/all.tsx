import React from 'react';
import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Delete Files By Session';

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
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function DeleteFiles({options}: Props) {
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
									initialValue: options.sessionId,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
