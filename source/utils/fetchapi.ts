import {useEffect, useState} from 'react';
import {getApiKey} from './session.js';
import {API_PATH} from './constants.js';

type Props = {
	method: string;
	endpoint: string;
	resultObject: string;
};

export function fetchApi({
	method,
	endpoint,
	resultObject,
}: Props): [boolean, any[]] {
	const [data, setData] = useState<any[]>([]);
	const [loading, setLoading] = useState<boolean>(true);

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
				const data = await response.json();
				setData(data[resultObject]);
			} catch (error) {
				console.error('Error fetching sessions:', error);
			}
			setLoading(false);
		}
		fetchData();
	}, []);

	return [loading, data];
}
