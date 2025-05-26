import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Sessions Endpoint';

export default function getSessions() {
	return (
		<ApiDashboard method="GET" endpoint="sessions" resultObject="sessions" />
	);
}
