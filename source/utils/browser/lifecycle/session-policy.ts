import {resolveBrowserAuth} from '../auth.js';
import {BrowserAdapterError} from '../errors.js';
import {DEFAULT_API_PATH} from './constants.js';
import type {
	BrowserSessionMode,
	BrowserSessionSummary,
	StartBrowserSessionOptions,
	UnknownRecord,
} from './types.js';

const CLOSED_SESSION_STATUSES = new Set([
	'closed',
	'completed',
	'ended',
	'failed',
	'released',
	'stopped',
	'terminated',
]);

type SessionRestartOptions = Pick<
	StartBrowserSessionOptions,
	| 'local'
	| 'apiUrl'
	| 'sessionName'
	| 'timeoutMs'
	| 'headless'
	| 'region'
	| 'solveCaptcha'
> & {apiUrl: string | null; sessionName: string | null};

function getConnectUrl(session: UnknownRecord): string | null {
	const candidateKeys = [
		'websocketUrl',
		'wsUrl',
		'connectUrl',
		'cdpUrl',
		'browserWSEndpoint',
		'wsEndpoint',
	];

	for (const key of candidateKeys) {
		const value = session[key];
		if (typeof value === 'string' && value.trim()) {
			return value.trim();
		}
	}

	return null;
}

function getViewerUrl(
	session: UnknownRecord,
	mode: BrowserSessionMode,
	sessionId: string,
): string | null {
	const candidateKeys = ['sessionViewerUrl', 'viewerUrl', 'liveViewUrl'];

	for (const key of candidateKeys) {
		const value = session[key];
		if (typeof value === 'string' && value.trim()) {
			return value.trim();
		}
	}

	if (mode === 'cloud') {
		return `https://app.steel.dev/sessions/${sessionId}`;
	}

	return null;
}

function getSessionStatus(session: UnknownRecord): string | null {
	const status = session['status'];
	if (typeof status === 'string' && status.trim()) {
		return status.trim();
	}

	return null;
}

export function getSessionId(session: UnknownRecord): string | null {
	const rawId = session['id'] ?? session['sessionId'];
	if (typeof rawId === 'string' && rawId.trim()) {
		return rawId.trim();
	}

	return null;
}

export function isSessionLive(session: UnknownRecord): boolean {
	const liveFlags = ['isLive', 'live', 'active'];

	for (const key of liveFlags) {
		const value = session[key];
		if (typeof value === 'boolean') {
			return value;
		}
	}

	const endedAt = session['endedAt'];
	if (typeof endedAt === 'string' && endedAt.trim()) {
		return false;
	}

	const status = getSessionStatus(session);
	if (status) {
		return !CLOSED_SESSION_STATUSES.has(status.toLowerCase());
	}

	return true;
}

export function toSessionSummary(
	session: UnknownRecord,
	mode: BrowserSessionMode,
	name: string | null,
	environment: NodeJS.ProcessEnv,
): BrowserSessionSummary {
	const sessionId = getSessionId(session);
	if (!sessionId) {
		throw new BrowserAdapterError(
			'API_ERROR',
			'Session response did not contain an id.',
		);
	}

	const auth = resolveBrowserAuth(environment);
	let resolvedConnectUrl =
		getConnectUrl(session) ||
		(mode === 'cloud' && auth.apiKey
			? `wss://connect.steel.dev?apiKey=${auth.apiKey}&sessionId=${sessionId}`
			: null);

	if (resolvedConnectUrl && mode === 'cloud' && auth.apiKey) {
		const parsed = new URL(resolvedConnectUrl);
		if (!parsed.searchParams.has('apiKey')) {
			parsed.searchParams.set('apiKey', auth.apiKey);
			resolvedConnectUrl = parsed.toString();
		}
	}

	return {
		id: sessionId,
		mode,
		name,
		live: isSessionLive(session),
		status: getSessionStatus(session),
		connectUrl: resolvedConnectUrl,
		viewerUrl: getViewerUrl(session, mode, sessionId),
		raw: session,
	};
}

function getApiErrorStatusCode(error: BrowserAdapterError): number | null {
	if (error.cause && typeof error.cause === 'object') {
		const status = (error.cause as UnknownRecord)['status'];
		if (typeof status === 'number' && Number.isInteger(status)) {
			return status;
		}
	}

	const statusCodeMatch = error.message.match(/\((\d{3})\)/);
	if (!statusCodeMatch?.[1]) {
		return null;
	}

	const parsedStatusCode = Number.parseInt(statusCodeMatch[1], 10);
	return Number.isInteger(parsedStatusCode) ? parsedStatusCode : null;
}

export function isNotFoundApiError(error: unknown): boolean {
	if (!(error instanceof BrowserAdapterError) || error.code !== 'API_ERROR') {
		return false;
	}

	const statusCode = getApiErrorStatusCode(error);
	return statusCode === 404 || statusCode === 410;
}

function isCloudApiUrl(
	apiUrl: string,
	environment: NodeJS.ProcessEnv,
): boolean {
	try {
		const explicitUrl = new URL(apiUrl);
		const configuredCloudApiUrl = new URL(
			environment.STEEL_API_URL?.trim() || DEFAULT_API_PATH,
		);
		return explicitUrl.origin === configuredCloudApiUrl.origin;
	} catch {
		return false;
	}
}

export function resolveSessionMode(
	localOption: boolean | undefined,
	apiUrl: string | null,
	activeSessionMode: BrowserSessionMode | null,
	environment: NodeJS.ProcessEnv,
): BrowserSessionMode {
	if (localOption) {
		return 'local';
	}

	if (apiUrl) {
		return isCloudApiUrl(apiUrl, environment) ? 'cloud' : 'local';
	}

	return activeSessionMode === 'local' ? 'local' : 'cloud';
}

function formatCliArgument(value: string): string {
	return /^[a-zA-Z0-9_./:=-]+$/.test(value) ? value : JSON.stringify(value);
}

function buildSessionRestartCommand(options: SessionRestartOptions): string {
	const args = ['steel browser start'];

	if (options.apiUrl) {
		args.push(`--api-url ${formatCliArgument(options.apiUrl)}`);
	} else if (options.local) {
		args.push('--local');
	}

	if (options.sessionName) {
		args.push(`--session ${formatCliArgument(options.sessionName)}`);
	}

	if (typeof options.timeoutMs === 'number') {
		args.push(`--session-timeout ${String(options.timeoutMs)}`);
	}

	if (options.headless) {
		args.push('--session-headless');
	}

	if (options.region) {
		args.push(`--session-region ${formatCliArgument(options.region)}`);
	}

	if (options.solveCaptcha) {
		args.push('--session-solve-captcha');
	}

	return args.join(' ');
}

export function buildDeadSessionMessage(
	sessionId: string,
	options: SessionRestartOptions,
): string {
	const sessionLabel = options.sessionName
		? `Mapped session "${options.sessionName}" (${sessionId})`
		: `Active session ${sessionId}`;
	const restartCommand = buildSessionRestartCommand(options);
	return `${sessionLabel} is no longer live. Run \`${restartCommand}\` to create a new session.`;
}
