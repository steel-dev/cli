import React from 'react';
import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Release Session By Id';

export const options = zod.object({
	id: zod
		.string()
		.uuid()
		.min(1)
		.max(36)
		.optional()
		.describe(
			option({
				description: 'The ID of the session to release',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function ReleaseSessionById({options}: Props) {
	return (
		<ApiDashboard
			method="POST"
			endpoint="sessions/{id}/release"
			form={{
				form: {
					title: 'Release Session By Id',
					sections: [
						{
							title: 'Session Details',
							fields: [
								{
									name: 'id',
									type: 'string',
									label: 'Session ID',
									required: true,
									initialValue: options.id,
								},
							],
						},
					],
				},
			}}
		/>
	);
}
