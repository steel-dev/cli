import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';

export default function getSessions() {
	return (
		<ApiDashboard method="GET" endpoint="sessions" resultObject="sessions" />
	);
}
