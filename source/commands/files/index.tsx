import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Get Files By Session';

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

export default function Files({options}: Props) {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{sessionId}/files"
			resultObject="data"
			form={{
				form: {
					title: 'Get Files By Session',
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
