import React from 'react';
import ApiDashboard from '../../../components/apidashboard.js';

export default function PDF() {
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
