import {useState, useCallback} from 'react';
import {getApiKey, getSettings} from '../utils/session.js';
import {API_PATH, LOCAL_API_PATH} from '../utils/constants.js';

type Options = {
	method: string;
	endpoint: string;
	contentType?: string;
	resultObject?: string;
};

type TriggerFn = (
	data?: Record<string, string>,
	callback?: (data: any[]) => void,
) => Promise<void>;

export function useLazyApi({
	method,
	endpoint,
	resultObject,
	contentType = 'application/json',
}: Options): [boolean, any[], Error | null, TriggerFn] {
	const [data, setData] = useState<any[]>([]);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<Error | null>(null);

	const trigger = useCallback<TriggerFn>(
		// what if I set it up so it first uses params and then adds the rest to body?
		// I think I can do a data param, first get params then the rest in body
		async (data, callback) => {
			setLoading(true);
			setError(null);

			try {
				const apiKey = getApiKey();
				if (!apiKey || !apiKey.apiKey || !apiKey.name) {
					throw new Error('API key not found');
				}
				const settings = getSettings();
				const url = settings?.instance === 'cloud' ? API_PATH : LOCAL_API_PATH;
				if (data) {
					endpoint = endpoint.replace(/\{(\w+)\}/g, (_, key: string) => {
						if (data[key]) {
							let value = data[key];
							delete data[key];
							return value;
						}
						return `{${key}}`;
					});
				}

				const response = await fetch(`${url}/${endpoint}`, {
					method,
					headers: {
						'Content-Type': contentType,
						'Steel-Api-Key': apiKey.apiKey,
					},
					body:
						data && Object.keys(data).length > 0
							? JSON.stringify(data)
							: undefined,
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
