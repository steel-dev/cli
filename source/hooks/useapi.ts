import {useEffect, useState} from 'react';
import {getApiKey} from '../utils/session.js';
import {API_PATH} from '../utils/constants.js';

type Props = {
	method: string;
	endpoint: string;
	resultObject?: string;
};

export function useApi({
	method,
	endpoint,
	resultObject,
}: Props): [boolean, any[], Error | null] {
	const [data, setData] = useState<any[]>([]);
	const [loading, setLoading] = useState<boolean>(true);
	const [error, setError] = useState<Error | null>(null);

	useEffect(() => {
		async function fetchData() {
			try {
				const apiKey = await getApiKey();
				if (!apiKey || !apiKey.apiKey || !apiKey.name) {
					throw new Error('API key not found');
				}
				const response = await fetch(`${API_PATH}/${endpoint}`, {
					method: method,
					headers: {
						'Content-Type': 'application/json',
						'Steel-Api-Key': apiKey?.apiKey,
					},
				});
				const json = await response.json();
				if (response.ok) {
					if (resultObject) {
						const result = json[resultObject];
						setData(result);
					} else {
						setData([json]);
					}
				} else {
					setError(new Error(json.message));
				}
			} catch (err) {
				console.error('API Error:', error);
				setError(err as Error);
			}
			setLoading(false);
		}
		fetchData();
	}, []);

	return [loading, data, error];
}
