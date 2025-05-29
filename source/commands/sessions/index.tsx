import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export const description = 'Get All Sessions';

export default function Sessions() {
	return (
		<ApiDashboard method="GET" endpoint="sessions" resultObject="sessions" />
	);
}
