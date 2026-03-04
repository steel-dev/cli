import {resolveBrowserAuth} from './browser/auth.js';
import {resolveCloudApiBaseUrl} from './browser/apiConfig.js';
import {BrowserAdapterError} from './browser/errors.js';

type UnknownRecord = Record<string, unknown>;

type CredentialValue = {
	username: string;
	password: string;
	totpSecret?: string;
};

type CreateCredentialOptions = {
	origin: string;
	value: CredentialValue;
	namespace?: string;
	label?: string;
};

type ListCredentialsOptions = {
	namespace?: string;
	origin?: string;
};

type UpdateCredentialOptions = {
	origin: string;
	value?: Partial<CredentialValue>;
	namespace?: string;
	label?: string;
};

type DeleteCredentialOptions = {
	origin: string;
	namespace?: string;
};

function getAuthHeaders(
	environment: NodeJS.ProcessEnv,
): Record<string, string> {
	const auth = resolveBrowserAuth(environment);
	if (!auth.apiKey) {
		throw new BrowserAdapterError(
			'MISSING_AUTH',
			'Missing API key. Run `steel login` or set `STEEL_API_KEY`.',
		);
	}

	return {
		'Content-Type': 'application/json',
		'Steel-Api-Key': auth.apiKey,
	};
}

async function requestCredentialsApi(
	environment: NodeJS.ProcessEnv,
	pathname: string,
	method: 'GET' | 'POST' | 'PUT' | 'DELETE',
	body?: UnknownRecord,
): Promise<unknown> {
	const headers = getAuthHeaders(environment);
	const apiBaseUrl = resolveCloudApiBaseUrl(environment);

	let response: Response;

	try {
		response = await fetch(`${apiBaseUrl}${pathname}`, {
			method,
			headers,
			body: body ? JSON.stringify(body) : undefined,
		});
	} catch (error) {
		throw new BrowserAdapterError(
			'API_ERROR',
			`Failed to reach Steel credentials API at ${apiBaseUrl}.`,
			error,
		);
	}

	const responseText = await response.text();
	let responseData: unknown = null;

	if (responseText.trim()) {
		try {
			responseData = JSON.parse(responseText);
		} catch {
			responseData = responseText;
		}
	}

	if (!response.ok) {
		const message =
			responseData &&
			typeof responseData === 'object' &&
			typeof (responseData as UnknownRecord)['message'] === 'string'
				? String((responseData as UnknownRecord)['message'])
				: response.statusText || 'Unknown API error';

		throw new BrowserAdapterError(
			'API_ERROR',
			`Steel credentials API request failed (${response.status}): ${message}`,
			{
				status: response.status,
				responseData,
			},
		);
	}

	return responseData;
}

export async function createCredential(
	options: CreateCredentialOptions,
	environment: NodeJS.ProcessEnv = process.env,
): Promise<UnknownRecord> {
	const payload: UnknownRecord = {
		origin: options.origin,
		value: {
			username: options.value.username,
			password: options.value.password,
			...(options.value.totpSecret
				? {totpSecret: options.value.totpSecret}
				: {}),
		},
	};

	if (options.namespace) {
		payload['namespace'] = options.namespace;
	}

	if (options.label) {
		payload['label'] = options.label;
	}

	const result = await requestCredentialsApi(
		environment,
		'/credentials',
		'POST',
		payload,
	);

	if (!result || typeof result !== 'object') {
		throw new BrowserAdapterError(
			'API_ERROR',
			'Unexpected empty response from Steel credentials API.',
		);
	}

	return result as UnknownRecord;
}

export async function listCredentials(
	options: ListCredentialsOptions = {},
	environment: NodeJS.ProcessEnv = process.env,
): Promise<UnknownRecord[]> {
	const params = new URLSearchParams();
	if (options.namespace) {
		params.set('namespace', options.namespace);
	}

	if (options.origin) {
		params.set('origin', options.origin);
	}

	const query = params.toString();
	const pathname = `/credentials${query ? `?${query}` : ''}`;

	const result = await requestCredentialsApi(environment, pathname, 'GET');

	if (Array.isArray(result)) {
		return result.filter(
			item => item && typeof item === 'object',
		) as UnknownRecord[];
	}

	if (result && typeof result === 'object') {
		const nested = (result as UnknownRecord)['credentials'];
		if (Array.isArray(nested)) {
			return nested.filter(
				item => item && typeof item === 'object',
			) as UnknownRecord[];
		}

		return [result as UnknownRecord];
	}

	return [];
}

export async function updateCredential(
	options: UpdateCredentialOptions,
	environment: NodeJS.ProcessEnv = process.env,
): Promise<UnknownRecord> {
	const payload: UnknownRecord = {};

	if (options.value) {
		const value: Record<string, string> = {};
		if (options.value.username) {
			value['username'] = options.value.username;
		}

		if (options.value.password) {
			value['password'] = options.value.password;
		}

		if (options.value.totpSecret) {
			value['totpSecret'] = options.value.totpSecret;
		}

		if (Object.keys(value).length > 0) {
			payload['value'] = value;
		}
	}

	if (options.label) {
		payload['label'] = options.label;
	}

	const params = new URLSearchParams();
	params.set('origin', options.origin);
	if (options.namespace) {
		params.set('namespace', options.namespace);
	}

	const query = params.toString();
	const result = await requestCredentialsApi(
		environment,
		`/credentials?${query}`,
		'PUT',
		payload,
	);

	if (!result || typeof result !== 'object') {
		throw new BrowserAdapterError(
			'API_ERROR',
			'Unexpected empty response from Steel credentials API.',
		);
	}

	return result as UnknownRecord;
}

export async function deleteCredential(
	options: DeleteCredentialOptions,
	environment: NodeJS.ProcessEnv = process.env,
): Promise<UnknownRecord> {
	const params = new URLSearchParams();
	params.set('origin', options.origin);
	if (options.namespace) {
		params.set('namespace', options.namespace);
	}

	const query = params.toString();
	const result = await requestCredentialsApi(
		environment,
		`/credentials?${query}`,
		'DELETE',
	);

	if (!result || typeof result !== 'object') {
		return {success: true};
	}

	return result as UnknownRecord;
}
