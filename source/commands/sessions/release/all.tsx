import ApiDashboard from '../../../components/apidashboard.js';

export const description = 'Release All Sessions';

export default function ReleaseAllSessions() {
	return (
		<ApiDashboard
			method="POST"
			endpoint="sessions/release"
			form={{
				form: {
					title: 'Release All Sessions',
					sections: [],
				},
			}}
		/>
	);
}
