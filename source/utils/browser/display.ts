const REDACTED_QUERY_VALUE = 'REDACTED';
const SENSITIVE_CONNECT_QUERY_KEYS = [
	'apiKey',
	'api_key',
	'token',
	'access_token',
];

export function sanitizeConnectUrlForDisplay(connectUrl: string): string {
	if (!connectUrl.trim()) {
		return connectUrl;
	}

	try {
		const parsed = new URL(connectUrl);
		let redacted = false;

		for (const key of SENSITIVE_CONNECT_QUERY_KEYS) {
			if (parsed.searchParams.has(key)) {
				parsed.searchParams.set(key, REDACTED_QUERY_VALUE);
				redacted = true;
			}
		}

		return redacted ? parsed.toString() : connectUrl;
	} catch {
		return connectUrl.replace(
			/([?&](?:apiKey|api_key|token|access_token)=)[^&]*/gi,
			`$1${REDACTED_QUERY_VALUE}`,
		);
	}
}
