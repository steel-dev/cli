import zod from 'zod';
import {option} from 'pastel';
import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Get Session Live Details By Id';
export const options = zod.object({
	id: zod
		.string()
		.uuid()
		.min(1)
		.max(36)
		.optional()
		.describe(
			option({
				description: 'The ID of the session to get live details',
			}),
		),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function SessionLiveDetailsById({options}: Props) {
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions/{id}/live-details"
			form={{
				form: {
					title: 'Get Session Live Details By Id',
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
