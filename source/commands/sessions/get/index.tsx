import zod from 'zod';
import ApiDashboard from '../../../components/apidashboard.js';
import {option} from 'pastel';

export const description = 'Get Session By Id';

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

export default function SessionsById({options}: Props) {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}"
			form={{
				form: {
					title: 'Get Session By Id',
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
