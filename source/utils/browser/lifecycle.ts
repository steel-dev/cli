import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {resolveBrowserAuth} from './auth.js';
import {BrowserAdapterError} from './errors.js';

export type BrowserSessionMode = 'cloud' | 'local';

type UnknownRecord = Record<string, unknown>;

type BrowserSessionState = {
	activeSessionId: string | null;
	activeSessionMode: BrowserSessionMode | null;
	activeSessionName: string | null;
	namedSessions: {
		cloud: Record<string, string>;
		local: Record<string, string>;
	};
	updatedAt: string | null;
};

type StartSessionRequestOptions = {
	stealth?: boolean;
	proxyUrl?: string;
};

type ParsedBootstrapOptions = {
	local: boolean;
	sessionName: string | null;
	stealth: boolean;
	proxyUrl: string | null;
	autoConnect: boolean;
	cdpTarget: string | null;
};

export type BrowserSessionSummary = {
	id: string;
	mode: BrowserSessionMode;
	name: string | null;
	live: boolean;
	status: string | null;
	connectUrl: string | null;
	viewerUrl: string | null;
	raw: UnknownRecord;
};

export type StartBrowserSessionOptions = {
	local?: boolean;
	sessionName?: string;
	stealth?: boolean;
	proxyUrl?: string;
	environment?: NodeJS.ProcessEnv;
};

export type StopBrowserSessionOptions = {
	all?: boolean;
	local?: boolean;
	environment?: NodeJS.ProcessEnv;
};

export type StopBrowserSessionResult = {
	mode: BrowserSessionMode;
	all: boolean;
	stoppedSessionIds: string[];
};

const DEFAULT_API_PATH = 'https://api.steel.dev/v1';
const DEFAULT_LOCAL_API_PATH = 'http://localhost:3000/v1';
const CONFIG_DIR =
	process.env.STEEL_CONFIG_DIR?.trim() ||
	path.join(os.homedir(), '.config', 'steel');
const SESSION_STATE_PATH = path.join(CONFIG_DIR, 'browser-session-state.json');
const SESSION_STATE_LOCK_PATH = `${SESSION_STATE_PATH}.lock`;
const LOCK_TIMEOUT_MS = 5000;
const LOCK_RETRY_MS = 50;
const LOCK_STALE_MS = 15_000;
const CLOSED_SESSION_STATUSES = new Set([
	'closed',
	'completed',
	'ended',
	'failed',
	'released',
	'stopped',
	'terminated',
]);

function wait(milliseconds: number): Promise<void> {
	return new Promise(resolve => {
		setTimeout(resolve, milliseconds);
	});
}

function getDefaultSessionState(): BrowserSessionState {
	return {
		activeSessionId: null,
		activeSessionMode: null,
		activeSessionName: null,
		namedSessions: {
			cloud: {},
			local: {},
		},
		updatedAt: null,
	};
}

function coerceSessionState(input: unknown): BrowserSessionState {
	const defaultState = getDefaultSessionState();
	if (!input || typeof input !== 'object') {
		return defaultState;
	}

	const state = input as Partial<BrowserSessionState>;

	const namedSessions =
		state.namedSessions && typeof state.namedSessions === 'object'
			? (state.namedSessions as Record<string, unknown>)
			: {};
	const cloudSessions =
		namedSessions && typeof namedSessions.cloud === 'object'
			? (namedSessions.cloud as Record<string, string>)
			: {};
	const localSessions =
		namedSessions && typeof namedSessions.local === 'object'
			? (namedSessions.local as Record<string, string>)
			: {};

	return {
		activeSessionId:
			typeof state.activeSessionId === 'string' ? state.activeSessionId : null,
		activeSessionMode:
			state.activeSessionMode === 'cloud' || state.activeSessionMode === 'local'
				? state.activeSessionMode
				: null,
		activeSessionName:
			typeof state.activeSessionName === 'string'
				? state.activeSessionName
				: null,
		namedSessions: {
			cloud: {...cloudSessions},
			local: {...localSessions},
		},
		updatedAt: typeof state.updatedAt === 'string' ? state.updatedAt : null,
	};
}

async function readSessionState(): Promise<BrowserSessionState> {
	try {
		const rawState = await fs.readFile(SESSION_STATE_PATH, 'utf-8');
		const parsedState = JSON.parse(rawState) as unknown;
		return coerceSessionState(parsedState);
	} catch {
		return getDefaultSessionState();
	}
}

async function writeSessionState(state: BrowserSessionState): Promise<void> {
	state.updatedAt = new Date().toISOString();
	await fs.mkdir(CONFIG_DIR, {recursive: true});
	await fs.writeFile(SESSION_STATE_PATH, JSON.stringify(state, null, 2));
}

async function releaseLock(): Promise<void> {
	try {
		await fs.unlink(SESSION_STATE_LOCK_PATH);
	} catch {
		// Ignore lock cleanup errors.
	}
}

async function acquireLock(): Promise<void> {
	const startedAt = Date.now();

	while (true) {
		try {
			const lockHandle = await fs.open(SESSION_STATE_LOCK_PATH, 'wx');
			await lockHandle.close();
			return;
		} catch (error) {
			const errorCode = (error as NodeJS.ErrnoException).code;
			if (errorCode !== 'EEXIST') {
				throw error;
			}

			try {
				const lockStat = await fs.stat(SESSION_STATE_LOCK_PATH);
				const lockAge = Date.now() - lockStat.mtimeMs;
				if (lockAge > LOCK_STALE_MS) {
					await releaseLock();
					continue;
				}
			} catch {
				// Another process may have released the lock.
			}

			if (Date.now() - startedAt >= LOCK_TIMEOUT_MS) {
				throw new BrowserAdapterError(
					'API_ERROR',
					'Timed out waiting for browser session state lock.',
				);
			}

			await wait(LOCK_RETRY_MS);
		}
	}
}

async function withSessionStateLock<T>(
	operation: (state: BrowserSessionState) => Promise<T>,
): Promise<T> {
	await fs.mkdir(CONFIG_DIR, {recursive: true});
	await acquireLock();

	try {
		const state = await readSessionState();
		const result = await operation(state);
		await writeSessionState(state);
		return result;
	} finally {
		await releaseLock();
	}
}

function setActiveSessionState(
	state: BrowserSessionState,
	mode: BrowserSessionMode,
	sessionId: string,
	sessionName: string | null,
): void {
	state.activeSessionMode = mode;
	state.activeSessionId = sessionId;
	state.activeSessionName = sessionName;
}

function clearActiveSessionState(
	state: BrowserSessionState,
	mode: BrowserSessionMode,
	sessionId: string,
): void {
	if (state.activeSessionMode === mode && state.activeSessionId === sessionId) {
		state.activeSessionMode = null;
		state.activeSessionId = null;
		state.activeSessionName = null;
	}

	for (const [name, mappedSessionId] of Object.entries(
		state.namedSessions[mode],
	)) {
		if (mappedSessionId === sessionId) {
			delete state.namedSessions[mode][name];
		}
	}
}

function normalizeApiBaseUrl(url: string): string {
	return url.replace(/\/+$/, '');
}

function getApiBaseUrl(
	mode: BrowserSessionMode,
	environment: NodeJS.ProcessEnv,
): string {
	if (mode === 'local') {
		return normalizeApiBaseUrl(
			environment.STEEL_LOCAL_API_URL?.trim() || DEFAULT_LOCAL_API_PATH,
		);
	}

	return normalizeApiBaseUrl(
		environment.STEEL_API_URL?.trim() || DEFAULT_API_PATH,
	);
}

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

function getSessionId(session: UnknownRecord): string | null {
	const rawId = session['id'] ?? session['sessionId'];
	if (typeof rawId === 'string' && rawId.trim()) {
		return rawId.trim();
	}

	return null;
}

function isSessionLive(session: UnknownRecord): boolean {
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

function toSessionSummary(
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

function extractSessionList(payload: unknown): UnknownRecord[] {
	if (Array.isArray(payload)) {
		return payload.filter(
			session => session && typeof session === 'object',
		) as UnknownRecord[];
	}

	if (!payload || typeof payload !== 'object') {
		return [];
	}

	const objectPayload = payload as UnknownRecord;
	const nestedSessions = objectPayload['sessions'];

	if (Array.isArray(nestedSessions)) {
		return nestedSessions.filter(
			session => session && typeof session === 'object',
		) as UnknownRecord[];
	}

	if (objectPayload['id']) {
		return [objectPayload];
	}

	return [];
}

function extractSingleSession(payload: unknown): UnknownRecord {
	if (payload && typeof payload === 'object') {
		const objectPayload = payload as UnknownRecord;
		const nestedSession = objectPayload['session'];

		if (nestedSession && typeof nestedSession === 'object') {
			return nestedSession as UnknownRecord;
		}

		return objectPayload;
	}

	throw new BrowserAdapterError(
		'API_ERROR',
		'Unexpected empty response from Steel session API.',
	);
}

async function requestApi(
	mode: BrowserSessionMode,
	environment: NodeJS.ProcessEnv,
	pathname: string,
	method: 'GET' | 'POST',
	body?: UnknownRecord,
): Promise<unknown> {
	const auth = resolveBrowserAuth(environment);
	if (mode === 'cloud' && !auth.apiKey) {
		throw new BrowserAdapterError(
			'MISSING_AUTH',
			'Missing browser auth. Run `steel login` or set `STEEL_API_KEY`.',
		);
	}

	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
	};

	if (auth.apiKey) {
		headers['Steel-Api-Key'] = auth.apiKey;
	}

	const response = await fetch(
		`${getApiBaseUrl(mode, environment)}${pathname}`,
		{
			method,
			headers,
			body: body ? JSON.stringify(body) : undefined,
		},
	);

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
			`Steel session API request failed (${response.status}): ${message}`,
		);
	}

	return responseData;
}

async function listSessionsFromApi(
	mode: BrowserSessionMode,
	environment: NodeJS.ProcessEnv,
): Promise<UnknownRecord[]> {
	const responseData = await requestApi(mode, environment, '/sessions', 'GET');
	return extractSessionList(responseData);
}

async function getSessionFromApi(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
): Promise<UnknownRecord> {
	const responseData = await requestApi(
		mode,
		environment,
		`/sessions/${sessionId}`,
		'GET',
	);
	return extractSingleSession(responseData);
}

async function createSessionFromApi(
	mode: BrowserSessionMode,
	options: StartSessionRequestOptions,
	environment: NodeJS.ProcessEnv,
): Promise<UnknownRecord> {
	const payload: UnknownRecord = {};

	if (options.proxyUrl?.trim()) {
		payload['proxyUrl'] = options.proxyUrl.trim();
	}

	if (options.stealth) {
		payload['stealthConfig'] = {
			humanizeInteractions: true,
			autoCaptchaSolving: true,
		};
		payload['solveCaptchas'] = true;
	}

	const responseData = await requestApi(
		mode,
		environment,
		'/sessions',
		'POST',
		payload,
	);
	return extractSingleSession(responseData);
}

async function releaseSession(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
): Promise<void> {
	await requestApi(mode, environment, `/sessions/${sessionId}/release`, 'POST');
}

async function tryGetLiveSession(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
): Promise<UnknownRecord | null> {
	try {
		const session = await getSessionFromApi(mode, sessionId, environment);
		return isSessionLive(session) ? session : null;
	} catch (error) {
		if (error instanceof BrowserAdapterError && error.code === 'API_ERROR') {
			return null;
		}

		throw error;
	}
}

function resolveNameFromState(
	state: BrowserSessionState,
	mode: BrowserSessionMode,
	sessionId: string,
): string | null {
	for (const [name, mappedSessionId] of Object.entries(
		state.namedSessions[mode],
	)) {
		if (mappedSessionId === sessionId) {
			return name;
		}
	}

	if (
		state.activeSessionMode === mode &&
		state.activeSessionId === sessionId &&
		state.activeSessionName
	) {
		return state.activeSessionName;
	}

	return null;
}

export function parseBrowserPassthroughBootstrapFlags(browserArgv: string[]): {
	options: ParsedBootstrapOptions;
	passthroughArgv: string[];
} {
	const options: ParsedBootstrapOptions = {
		local: false,
		sessionName: null,
		stealth: false,
		proxyUrl: null,
		autoConnect: false,
		cdpTarget: null,
	};
	const passthroughArgv: string[] = [];

	for (let index = 0; index < browserArgv.length; index++) {
		const argument = browserArgv[index];

		if (argument === '--local') {
			options.local = true;
			continue;
		}

		if (argument === '--stealth') {
			options.stealth = true;
			continue;
		}

		if (argument === '--session' || argument.startsWith('--session=')) {
			const value =
				argument === '--session'
					? browserArgv[index + 1]
					: argument.slice('--session='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session.',
				);
			}

			options.sessionName = value.trim();

			if (argument === '--session') {
				index++;
			}

			continue;
		}

		if (argument === '--proxy' || argument.startsWith('--proxy=')) {
			const value =
				argument === '--proxy'
					? browserArgv[index + 1]
					: argument.slice('--proxy='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --proxy.',
				);
			}

			options.proxyUrl = value.trim();

			if (argument === '--proxy') {
				index++;
			}

			continue;
		}

		if (argument === '--auto-connect') {
			const potentialValue = browserArgv[index + 1];
			if (potentialValue && !potentialValue.startsWith('-')) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'`--auto-connect` does not accept a value. Use `--cdp <url|port>` for explicit endpoints.',
				);
			}

			options.autoConnect = true;
			passthroughArgv.push('--auto-connect');
			continue;
		}

		if (argument.startsWith('--auto-connect=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--auto-connect` does not accept a value. Use `--cdp <url|port>` for explicit endpoints.',
			);
		}

		if (argument === '--cdp' || argument.startsWith('--cdp=')) {
			const value =
				argument === '--cdp'
					? browserArgv[index + 1]
					: argument.slice('--cdp='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --cdp.',
				);
			}

			options.cdpTarget = value.trim();
			passthroughArgv.push('--cdp', value.trim());

			if (argument === '--cdp') {
				index++;
			}

			continue;
		}

		passthroughArgv.push(argument);
	}

	if (options.autoConnect && options.cdpTarget) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'Cannot combine `--auto-connect` with `--cdp`.',
		);
	}

	return {
		options,
		passthroughArgv,
	};
}

export async function startBrowserSession(
	options: StartBrowserSessionOptions = {},
): Promise<BrowserSessionSummary> {
	const mode: BrowserSessionMode = options.local ? 'local' : 'cloud';
	const environment = options.environment || process.env;
	const sessionName = options.sessionName?.trim() || null;

	return withSessionStateLock(async state => {
		let candidateSessionId: string | null = null;

		if (sessionName) {
			candidateSessionId = state.namedSessions[mode][sessionName] || null;
		} else if (state.activeSessionMode === mode && state.activeSessionId) {
			candidateSessionId = state.activeSessionId;
		}

		if (candidateSessionId) {
			const existingSession = await tryGetLiveSession(
				mode,
				candidateSessionId,
				environment,
			);

			if (existingSession) {
				setActiveSessionState(state, mode, candidateSessionId, sessionName);
				return toSessionSummary(
					existingSession,
					mode,
					sessionName,
					environment,
				);
			}

			clearActiveSessionState(state, mode, candidateSessionId);
		}

		const createdSession = await createSessionFromApi(
			mode,
			{
				stealth: options.stealth,
				proxyUrl: options.proxyUrl,
			},
			environment,
		);
		const createdSessionId = getSessionId(createdSession);

		if (!createdSessionId) {
			throw new BrowserAdapterError(
				'API_ERROR',
				'Failed to create a session because the API did not return a session id.',
			);
		}

		if (sessionName) {
			state.namedSessions[mode][sessionName] = createdSessionId;
		}

		setActiveSessionState(state, mode, createdSessionId, sessionName);
		return toSessionSummary(createdSession, mode, sessionName, environment);
	});
}

export async function listBrowserSessions(options?: {
	local?: boolean;
	environment?: NodeJS.ProcessEnv;
}): Promise<BrowserSessionSummary[]> {
	const mode: BrowserSessionMode = options?.local ? 'local' : 'cloud';
	const environment = options?.environment || process.env;
	const state = await readSessionState();
	const sessions = await listSessionsFromApi(mode, environment);

	return sessions.map(session => {
		const sessionId = getSessionId(session);
		const sessionName = sessionId
			? resolveNameFromState(state, mode, sessionId)
			: null;
		return toSessionSummary(session, mode, sessionName, environment);
	});
}

export async function getActiveBrowserLiveUrl(options?: {
	local?: boolean;
	environment?: NodeJS.ProcessEnv;
}): Promise<string | null> {
	const environment = options?.environment || process.env;
	const state = await readSessionState();
	const mode: BrowserSessionMode =
		options?.local || (!options?.local && state.activeSessionMode === 'local')
			? 'local'
			: 'cloud';

	if (state.activeSessionMode !== mode || !state.activeSessionId) {
		return null;
	}

	const session = await tryGetLiveSession(
		mode,
		state.activeSessionId,
		environment,
	);
	if (!session) {
		return null;
	}

	const summary = toSessionSummary(
		session,
		mode,
		state.activeSessionName,
		environment,
	);
	return summary.viewerUrl;
}

export async function stopBrowserSession(
	options: StopBrowserSessionOptions = {},
): Promise<StopBrowserSessionResult> {
	const environment = options.environment || process.env;

	return withSessionStateLock(async state => {
		const mode: BrowserSessionMode =
			options.local || (!options.local && state.activeSessionMode === 'local')
				? 'local'
				: 'cloud';

		if (options.all) {
			const sessions = await listSessionsFromApi(mode, environment);
			const liveSessionIds = sessions
				.filter(session => isSessionLive(session))
				.map(session => getSessionId(session))
				.filter((sessionId): sessionId is string => Boolean(sessionId));

			for (const sessionId of liveSessionIds) {
				try {
					await releaseSession(mode, sessionId, environment);
				} catch {
					// Continue best-effort release for all sessions.
				}

				clearActiveSessionState(state, mode, sessionId);
			}

			return {
				mode,
				all: true,
				stoppedSessionIds: liveSessionIds,
			};
		}

		const targetSessionId =
			state.activeSessionMode === mode ? state.activeSessionId : null;

		if (!targetSessionId) {
			return {
				mode,
				all: false,
				stoppedSessionIds: [],
			};
		}

		await releaseSession(mode, targetSessionId, environment);
		clearActiveSessionState(state, mode, targetSessionId);

		return {
			mode,
			all: false,
			stoppedSessionIds: [targetSessionId],
		};
	});
}

export async function bootstrapBrowserPassthroughArgv(
	browserArgv: string[],
	environment: NodeJS.ProcessEnv = process.env,
): Promise<string[]> {
	const parsed = parseBrowserPassthroughBootstrapFlags(browserArgv);

	if (parsed.options.autoConnect || parsed.options.cdpTarget) {
		return parsed.passthroughArgv;
	}

	if (
		parsed.passthroughArgv.includes('--help') ||
		parsed.passthroughArgv.includes('-h')
	) {
		return parsed.passthroughArgv;
	}

	const session = await startBrowserSession({
		local: parsed.options.local,
		sessionName: parsed.options.sessionName || undefined,
		stealth: parsed.options.stealth,
		proxyUrl: parsed.options.proxyUrl || undefined,
		environment,
	});

	if (!session.connectUrl) {
		throw new BrowserAdapterError(
			'API_ERROR',
			`Could not resolve a connect URL for session ${session.id}.`,
		);
	}

	return [...parsed.passthroughArgv, '--cdp', session.connectUrl];
}
