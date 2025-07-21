import {useEffect, useState} from 'react';
import {getApiKey, getSettings} from '../utils/session.js';
import {API_PATH, LOCAL_API_PATH} from '../utils/constants.js';

type Props = {
	method: string;
	endpoint: string;
	resultObject?: string;
};

export function useApi({
	method,
	endpoint,
	resultObject,
}: Props): [boolean, object[], Error | null] {
	const [data, setData] = useState<object[]>([]);
	const [loading, setLoading] = useState<boolean>(true);
	const [error, setError] = useState<Error | null>(null);

	useEffect(() => {
		async function fetchData() {
			try {
				const apiKey = getApiKey();

				if (!apiKey || !apiKey.apiKey || !apiKey.name) {
					throw new Error('API key not found');
				}

				const settings = getSettings();

				const url = settings?.instance === 'cloud' ? API_PATH : LOCAL_API_PATH;

				const response = await fetch(`${url}/${endpoint}`, {
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
						// setData([json]);
						setData(json);
					}
				} else {
					setError(new Error(json.message));
				}
			} catch (err) {
				setError(err as Error);
			}
			setLoading(false);
		}
		fetchData();
	}, []);

	return [loading, data, error];
}
