import React from 'react';
import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Get Session Context By Id';

export const options = zod.object({
	id: zod
		.string()
		.uuid()
		.min(1)
		.max(36)
		.optional()
		.describe(
			option({
				description: 'The ID of the session to retrieve',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function SessionContextById({options}: Props) {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}/context"
			form={{
				form: {
					title: 'Get Session Context By Id',
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
