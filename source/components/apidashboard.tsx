import React, {useState} from 'react';
import PostDashboard from './postdashboard.js';
import GetDashboard from './getdashboard.js';
import {useLazyApi} from '../hooks/uselazyapi.js';
import {useApi} from '../hooks/useapi.js';
import {FormProps} from 'ink-form';

type Props = {
	form?: FormProps;
	method: string;
	endpoint: string;
	path_params?: object[];
	resultObject?: string; // Where the results of the API call are stored on the object
};

export default function ApiDashboard({
	form,
	method,
	endpoint,
	path_params,
	resultObject,
}: Props) {
	if (method === 'POST' && form && form.form.sections.length > 0) {
		const [loading, data, error, callback] = useLazyApi({
			method,
			endpoint,
			resultObject,
		});
		const [sent, setSent] = useState(false);
		return sent ? (
			<GetDashboard
				data={data}
				loading={loading}
				error={error}
				method={method}
				endpoint={endpoint}
			/>
		) : (
			<PostDashboard
				form={form}
				callback={result => {
					callback(result);
					setSent(true);
				}}
			/>
		);
	} else if (
		method === 'GET' ||
		(method === 'POST' && form && form.form.sections.length === 0)
	) {
		const [loading, data, error] = useApi({
			method,
			endpoint,
			resultObject,
		});
		return (
			<GetDashboard
				data={data}
				loading={loading}
				error={error}
				method={method}
				endpoint={endpoint}
			/>
		);
	} else return null;
}
