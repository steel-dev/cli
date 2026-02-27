import type {BrowserSessionSummary} from './lifecycle.js';

const REDACTED_QUERY_VALUE = 'REDACTED';
const SENSITIVE_CONNECT_QUERY_KEYS = [
	'apiKey',
	'api_key',
	'token',
	'access_token',
];

export type BrowserSessionsOutputSession = Omit<
	BrowserSessionSummary,
	'raw'
> & {
	raw?: BrowserSessionSummary['raw'];
};

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

export function formatBrowserSessionsForOutput(
	sessions: BrowserSessionSummary[],
	options: {raw?: boolean} = {},
): {sessions: BrowserSessionsOutputSession[]} {
	return {
		sessions: sessions.map(session => {
			const summary: BrowserSessionsOutputSession = {
				id: session.id,
				mode: session.mode,
				name: session.name,
				live: session.live,
				status: session.status,
				connectUrl:
					typeof session.connectUrl === 'string'
						? sanitizeConnectUrlForDisplay(session.connectUrl)
						: session.connectUrl,
				viewerUrl: session.viewerUrl,
			};

			if (options.raw) {
				summary.raw = session.raw;
			}

			return summary;
		}),
	};
}
