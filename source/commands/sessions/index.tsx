import React from 'react';
import ApiDashboard from '../../components/apidashboard.js';
import {getSettings} from '../../utils/session.js';

export const description = 'Get All Sessions';

export default function Sessions() {
	const settings = getSettings();
	return (
		<ApiDashboard
			method="GET"
			endpoint="sessions"
			resultObject={settings?.instance === 'cloud' ? 'sessions' : undefined}
		/>
	);
}
