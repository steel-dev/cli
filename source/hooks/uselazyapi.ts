import {useState, useCallback} from 'react';
import {getApiKey} from '../utils/session.js';
import {API_PATH} from '../utils/constants.js';

type Options = {
	method: string;
	endpoint: string;
	resultObject?: string;
};

type TriggerFn = (
	body?: any,
	callback?: (data: any[]) => void,
) => Promise<void>;

export function useLazyApi({
	method,
	endpoint,
	resultObject,
}: Options): [boolean, any[], Error | null, TriggerFn] {
	const [data, setData] = useState<any[]>([]);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<Error | null>(null);

	const trigger = useCallback<TriggerFn>(
		async (body, callback) => {
			setLoading(true);
			setError(null);

			try {
				const apiKey = await getApiKey();
				if (!apiKey?.apiKey) throw new Error('API key not found');

				const response = await fetch(`${API_PATH}/${endpoint}`, {
					method,
					headers: {
						'Content-Type': 'application/json',
						'Steel-Api-Key': apiKey.apiKey,
					},
					body: body ? JSON.stringify(body) : undefined,
				});

				const json = await response.json();

				if (response.ok) {
					if (resultObject) {
						const result = json[resultObject];
						setData(result);
						if (callback) callback(result);
					} else {
						setData([json]);
						if (callback) callback([json]);
					}
				} else {
					setError(new Error(json.message));
				}
			} catch (err) {
				console.error('API Error:', err);
				setError(err as Error);
			}

			setLoading(false);
		},
		[method, endpoint, resultObject],
	);

	return [loading, data, error, trigger];
}
