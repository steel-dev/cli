import * as fsSync from 'node:fs';
import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {getLocalBrowserRepoPath} from '../dev/local.js';
import {resolveBrowserAuth} from './auth.js';
import {BrowserAdapterError} from './errors.js';

export type BrowserSessionMode = 'cloud' | 'local';
type DeadSessionBehavior = 'recreate' | 'error';

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
	timeoutMs?: number;
	headless?: boolean;
	region?: string;
	solveCaptcha?: boolean;
};

type ParsedBootstrapOptions = {
	local: boolean;
	apiUrl: string | null;
	sessionName: string | null;
	stealth: boolean;
	proxyUrl: string | null;
	timeoutMs: number | null;
	headless: boolean;
	region: string | null;
	solveCaptcha: boolean;
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
	apiUrl?: string;
	sessionName?: string;
	stealth?: boolean;
	proxyUrl?: string;
	timeoutMs?: number;
	headless?: boolean;
	region?: string;
	solveCaptcha?: boolean;
	deadSessionBehavior?: DeadSessionBehavior;
	environment?: NodeJS.ProcessEnv;
};

export type StopBrowserSessionOptions = {
	all?: boolean;
	local?: boolean;
	apiUrl?: string;
	environment?: NodeJS.ProcessEnv;
};

type BrowserSessionEndpointOptions = {
	local?: boolean;
	apiUrl?: string;
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
	options?: {write?: boolean},
): Promise<T> {
	await fs.mkdir(CONFIG_DIR, {recursive: true});
	await acquireLock();

	try {
		const state = await readSessionState();
		const result = await operation(state);
		if (options?.write !== false) {
			await writeSessionState(state);
		}
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

function getConfigDirectory(environment: NodeJS.ProcessEnv): string {
	return (
		environment.STEEL_CONFIG_DIR?.trim() ||
		path.join(os.homedir(), '.config', 'steel')
	);
}

function getConfigPath(environment: NodeJS.ProcessEnv): string {
	return path.join(getConfigDirectory(environment), 'config.json');
}

function coerceLocalApiUrlFromConfig(config: unknown): string | null {
	if (!config || typeof config !== 'object') {
		return null;
	}

	const browser = (config as Record<string, unknown>)['browser'];
	if (!browser || typeof browser !== 'object') {
		return null;
	}

	const apiUrl = (browser as Record<string, unknown>)['apiUrl'];
	if (typeof apiUrl === 'string' && apiUrl.trim()) {
		return apiUrl.trim();
	}

	return null;
}

async function getLocalApiUrlFromConfig(
	environment: NodeJS.ProcessEnv,
): Promise<string | null> {
	try {
		const configPath = getConfigPath(environment);
		const configContents = await fs.readFile(configPath, 'utf-8');
		const parsedConfig = JSON.parse(configContents) as unknown;
		return coerceLocalApiUrlFromConfig(parsedConfig);
	} catch {
		return null;
	}
}

function validateApiUrl(url: string): string {
	try {
		return normalizeApiBaseUrl(new URL(url).toString());
	} catch {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid value for --api-url: ${url}`,
		);
	}
}

function resolveExplicitApiUrl(apiUrl?: string | null): string | null {
	if (apiUrl === undefined || apiUrl === null) {
		return null;
	}

	const trimmedApiUrl = apiUrl.trim();
	if (!trimmedApiUrl) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'Missing value for --api-url.',
		);
	}

	return validateApiUrl(trimmedApiUrl);
}

function parsePositiveIntegerFlagValue(
	value: string,
	flagName: string,
): number {
	const parsed = Number.parseInt(value, 10);
	if (!Number.isSafeInteger(parsed) || parsed <= 0) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid value for ${flagName}: ${value}. Expected a positive integer.`,
		);
	}

	return parsed;
}

async function getApiBaseUrl(
	mode: BrowserSessionMode,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<string> {
	const explicitApiUrl = resolveExplicitApiUrl(apiUrl);
	if (explicitApiUrl) {
		return explicitApiUrl;
	}

	if (mode === 'local') {
		const envApiUrl =
			environment.STEEL_BROWSER_API_URL?.trim() ||
			environment.STEEL_LOCAL_API_URL?.trim();
		if (envApiUrl) {
			return normalizeApiBaseUrl(envApiUrl);
		}

		const configApiUrl = await getLocalApiUrlFromConfig(environment);
		if (configApiUrl) {
			return normalizeApiBaseUrl(configApiUrl);
		}

		return normalizeApiBaseUrl(DEFAULT_LOCAL_API_PATH);
	}

	return normalizeApiBaseUrl(
		environment.STEEL_API_URL?.trim() || DEFAULT_API_PATH,
	);
}

function isLocalhostEndpoint(apiBaseUrl: string): boolean {
	try {
		const parsedUrl = new URL(apiBaseUrl);
		return (
			parsedUrl.hostname === 'localhost' ||
			parsedUrl.hostname === '127.0.0.1' ||
			parsedUrl.hostname === '::1'
		);
	} catch {
		return false;
	}
}

function isLocalRuntimeInstalled(environment: NodeJS.ProcessEnv): boolean {
	const repoPath = getLocalBrowserRepoPath(getConfigDirectory(environment));
	return fsSync.existsSync(repoPath);
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
	apiUrl?: string | null,
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

	const apiBaseUrl = await getApiBaseUrl(mode, environment, apiUrl);
	let response: Response;

	try {
		response = await fetch(`${apiBaseUrl}${pathname}`, {
			method,
			headers,
			body: body ? JSON.stringify(body) : undefined,
		});
	} catch (error) {
		if (mode === 'local' && isLocalhostEndpoint(apiBaseUrl)) {
			if (!isLocalRuntimeInstalled(environment)) {
				throw new BrowserAdapterError(
					'API_ERROR',
					'Local Steel Browser runtime is not installed. Run `steel dev install` first.',
					error,
				);
			}

			throw new BrowserAdapterError(
				'API_ERROR',
				'Local Steel Browser runtime is not running. Run `steel dev start` and try again.',
				error,
			);
		}

		throw new BrowserAdapterError(
			'API_ERROR',
			`Failed to reach Steel session API at ${apiBaseUrl}.`,
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
			`Steel session API request failed (${response.status}): ${message}`,
			{
				status: response.status,
				responseData,
			},
		);
	}

	return responseData;
}

async function listSessionsFromApi(
	mode: BrowserSessionMode,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord[]> {
	const responseData = await requestApi(
		mode,
		environment,
		'/sessions',
		'GET',
		undefined,
		apiUrl,
	);
	return extractSessionList(responseData);
}

async function getSessionFromApi(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord> {
	const responseData = await requestApi(
		mode,
		environment,
		`/sessions/${sessionId}`,
		'GET',
		undefined,
		apiUrl,
	);
	return extractSingleSession(responseData);
}

async function createSessionFromApi(
	mode: BrowserSessionMode,
	options: StartSessionRequestOptions,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord> {
	const payload: UnknownRecord = {};

	if (options.proxyUrl?.trim()) {
		payload['proxyUrl'] = options.proxyUrl.trim();
	}

	if (typeof options.timeoutMs === 'number') {
		payload['timeout'] = options.timeoutMs;
	}

	if (typeof options.headless === 'boolean') {
		payload['headless'] = options.headless;
	}

	if (options.region?.trim()) {
		payload['region'] = options.region.trim();
	}

	if (options.stealth) {
		payload['stealthConfig'] = {
			humanizeInteractions: true,
			autoCaptchaSolving: true,
		};
		payload['solveCaptcha'] = true;
	}

	if (options.solveCaptcha) {
		payload['solveCaptcha'] = true;
	}

	const responseData = await requestApi(
		mode,
		environment,
		'/sessions',
		'POST',
		payload,
		apiUrl,
	);
	return extractSingleSession(responseData);
}

async function releaseSession(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<void> {
	await requestApi(
		mode,
		environment,
		`/sessions/${sessionId}/release`,
		'POST',
		undefined,
		apiUrl,
	);
}

async function tryGetLiveSession(
	mode: BrowserSessionMode,
	sessionId: string,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord | null> {
	try {
		const session = await getSessionFromApi(
			mode,
			sessionId,
			environment,
			apiUrl,
		);
		return isSessionLive(session) ? session : null;
	} catch (error) {
		if (isNotFoundApiError(error)) {
			return null;
		}

		throw error;
	}
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

function isNotFoundApiError(error: unknown): boolean {
	if (!(error instanceof BrowserAdapterError) || error.code !== 'API_ERROR') {
		return false;
	}

	const statusCode = getApiErrorStatusCode(error);
	return statusCode === 404 || statusCode === 410;
}

function resolveCandidateSessionId(
	state: BrowserSessionState,
	mode: BrowserSessionMode,
	sessionName: string | null,
): string | null {
	if (sessionName) {
		return state.namedSessions[mode][sessionName] || null;
	}

	if (state.activeSessionMode === mode && state.activeSessionId) {
		return state.activeSessionId;
	}

	return null;
}

function resolveSessionMode(
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

function formatCliArgument(value: string): string {
	return /^[a-zA-Z0-9_./:=-]+$/.test(value) ? value : JSON.stringify(value);
}

function buildSessionRestartCommand(
	options: Pick<
		StartBrowserSessionOptions,
		| 'local'
		| 'apiUrl'
		| 'sessionName'
		| 'timeoutMs'
		| 'headless'
		| 'region'
		| 'solveCaptcha'
	> & {apiUrl: string | null; sessionName: string | null},
): string {
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

function buildDeadSessionMessage(
	sessionId: string,
	options: Pick<
		StartBrowserSessionOptions,
		| 'local'
		| 'apiUrl'
		| 'sessionName'
		| 'timeoutMs'
		| 'headless'
		| 'region'
		| 'solveCaptcha'
	> & {apiUrl: string | null; sessionName: string | null},
): string {
	const sessionLabel = options.sessionName
		? `Mapped session "${options.sessionName}" (${sessionId})`
		: `Active session ${sessionId}`;
	const restartCommand = buildSessionRestartCommand(options);
	return `${sessionLabel} is no longer live. Run \`${restartCommand}\` to create a new session.`;
}

export function parseBrowserPassthroughBootstrapFlags(browserArgv: string[]): {
	options: ParsedBootstrapOptions;
	passthroughArgv: string[];
} {
	const options: ParsedBootstrapOptions = {
		local: false,
		apiUrl: null,
		sessionName: null,
		stealth: false,
		proxyUrl: null,
		timeoutMs: null,
		headless: false,
		region: null,
		solveCaptcha: false,
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

		if (argument === '--api-url' || argument.startsWith('--api-url=')) {
			const value =
				argument === '--api-url'
					? browserArgv[index + 1]
					: argument.slice('--api-url='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --api-url.',
				);
			}

			options.apiUrl = resolveExplicitApiUrl(value);

			if (argument === '--api-url') {
				index++;
			}

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

		if (
			argument === '--session-timeout' ||
			argument.startsWith('--session-timeout=')
		) {
			const value =
				argument === '--session-timeout'
					? browserArgv[index + 1]
					: argument.slice('--session-timeout='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-timeout.',
				);
			}

			options.timeoutMs = parsePositiveIntegerFlagValue(
				value,
				'--session-timeout',
			);

			if (argument === '--session-timeout') {
				index++;
			}

			continue;
		}

		if (argument === '--session-headless') {
			options.headless = true;
			continue;
		}

		if (argument.startsWith('--session-headless=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--session-headless` does not accept a value.',
			);
		}

		if (
			argument === '--session-region' ||
			argument.startsWith('--session-region=')
		) {
			const value =
				argument === '--session-region'
					? browserArgv[index + 1]
					: argument.slice('--session-region='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-region.',
				);
			}

			const normalizedRegion = value.trim();
			if (!normalizedRegion) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-region.',
				);
			}

			options.region = normalizedRegion;

			if (argument === '--session-region') {
				index++;
			}

			continue;
		}

		if (argument === '--session-solve-captcha') {
			options.solveCaptcha = true;
			continue;
		}

		if (argument.startsWith('--session-solve-captcha=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--session-solve-captcha` does not accept a value.',
			);
		}

		if (argument === '--auto-connect') {
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
	const environment = options.environment || process.env;
	const apiUrl = resolveExplicitApiUrl(options.apiUrl);
	const mode = resolveSessionMode(options.local, apiUrl, null, environment);
	const sessionName = options.sessionName?.trim() || null;
	const deadSessionBehavior = options.deadSessionBehavior || 'recreate';

	while (true) {
		const candidateSessionId = await withSessionStateLock(
			async state => resolveCandidateSessionId(state, mode, sessionName),
			{write: false},
		);

		if (candidateSessionId) {
			const existingSession = await tryGetLiveSession(
				mode,
				candidateSessionId,
				environment,
				apiUrl,
			);

			if (existingSession) {
				const claimedExistingSession = await withSessionStateLock(
					async state => {
						const latestCandidateSessionId = resolveCandidateSessionId(
							state,
							mode,
							sessionName,
						);
						if (latestCandidateSessionId !== candidateSessionId) {
							return false;
						}

						setActiveSessionState(state, mode, candidateSessionId, sessionName);
						return true;
					},
				);

				if (claimedExistingSession) {
					return toSessionSummary(
						existingSession,
						mode,
						sessionName,
						environment,
					);
				}

				continue;
			}

			if (deadSessionBehavior === 'error') {
				throw new BrowserAdapterError(
					'SESSION_NOT_FOUND',
					buildDeadSessionMessage(candidateSessionId, {
						local: options.local,
						apiUrl,
						sessionName,
						timeoutMs: options.timeoutMs,
						headless: options.headless,
						region: options.region,
						solveCaptcha: options.solveCaptcha,
					}),
				);
			}

			await withSessionStateLock(async state => {
				const latestCandidateSessionId = resolveCandidateSessionId(
					state,
					mode,
					sessionName,
				);
				if (latestCandidateSessionId === candidateSessionId) {
					clearActiveSessionState(state, mode, candidateSessionId);
				}
			});
			continue;
		}

		const createdSession = await createSessionFromApi(
			mode,
			{
				stealth: options.stealth,
				proxyUrl: options.proxyUrl,
				timeoutMs: options.timeoutMs,
				headless: options.headless,
				region: options.region,
				solveCaptcha: options.solveCaptcha,
			},
			environment,
			apiUrl,
		);
		const createdSessionId = getSessionId(createdSession);

		if (!createdSessionId) {
			throw new BrowserAdapterError(
				'API_ERROR',
				'Failed to create a session because the API did not return a session id.',
			);
		}

		const claimedCreatedSession = await withSessionStateLock(async state => {
			const latestCandidateSessionId = resolveCandidateSessionId(
				state,
				mode,
				sessionName,
			);
			if (latestCandidateSessionId) {
				return false;
			}

			if (sessionName) {
				state.namedSessions[mode][sessionName] = createdSessionId;
			}
			setActiveSessionState(state, mode, createdSessionId, sessionName);
			return true;
		});

		if (claimedCreatedSession) {
			return toSessionSummary(createdSession, mode, sessionName, environment);
		}

		try {
			await releaseSession(mode, createdSessionId, environment, apiUrl);
		} catch {
			// Continue with the state-selected session if cleanup fails.
		}
	}
}

export async function listBrowserSessions(
	options?: BrowserSessionEndpointOptions,
): Promise<BrowserSessionSummary[]> {
	const environment = options?.environment || process.env;
	const apiUrl = resolveExplicitApiUrl(options?.apiUrl);
	const mode = resolveSessionMode(options?.local, apiUrl, null, environment);
	const state = await readSessionState();
	const sessions = await listSessionsFromApi(mode, environment, apiUrl);

	return sessions.map(session => {
		const sessionId = getSessionId(session);
		const sessionName = sessionId
			? resolveNameFromState(state, mode, sessionId)
			: null;
		return toSessionSummary(session, mode, sessionName, environment);
	});
}

export async function getActiveBrowserLiveUrl(
	options?: BrowserSessionEndpointOptions,
): Promise<string | null> {
	const environment = options?.environment || process.env;
	const apiUrl = resolveExplicitApiUrl(options?.apiUrl);
	const state = await readSessionState();
	const mode = resolveSessionMode(
		options?.local,
		apiUrl,
		state.activeSessionMode,
		environment,
	);

	if (state.activeSessionMode !== mode || !state.activeSessionId) {
		return null;
	}

	const session = await tryGetLiveSession(
		mode,
		state.activeSessionId,
		environment,
		apiUrl,
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
	const apiUrl = resolveExplicitApiUrl(options.apiUrl);
	const initialState = await withSessionStateLock(
		async state => {
			const mode = resolveSessionMode(
				options.local,
				apiUrl,
				state.activeSessionMode,
				environment,
			);

			return {
				mode,
				targetSessionId:
					state.activeSessionMode === mode ? state.activeSessionId : null,
			};
		},
		{write: false},
	);
	const mode = initialState.mode;

	if (options.all) {
		const sessions = await listSessionsFromApi(mode, environment, apiUrl);
		const liveSessionIds = sessions
			.filter(session => isSessionLive(session))
			.map(session => getSessionId(session))
			.filter((sessionId): sessionId is string => Boolean(sessionId));

		for (const sessionId of liveSessionIds) {
			try {
				await releaseSession(mode, sessionId, environment, apiUrl);
			} catch {
				// Continue best-effort release for all sessions.
			}
		}

		await withSessionStateLock(async state => {
			for (const sessionId of liveSessionIds) {
				clearActiveSessionState(state, mode, sessionId);
			}
		});

		return {
			mode,
			all: true,
			stoppedSessionIds: liveSessionIds,
		};
	}

	const targetSessionId = initialState.targetSessionId;
	if (!targetSessionId) {
		return {
			mode,
			all: false,
			stoppedSessionIds: [],
		};
	}

	await releaseSession(mode, targetSessionId, environment, apiUrl);
	await withSessionStateLock(async state => {
		clearActiveSessionState(state, mode, targetSessionId);
	});

	return {
		mode,
		all: false,
		stoppedSessionIds: [targetSessionId],
	};
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
		apiUrl: parsed.options.apiUrl || undefined,
		sessionName: parsed.options.sessionName || undefined,
		stealth: parsed.options.stealth,
		proxyUrl: parsed.options.proxyUrl || undefined,
		timeoutMs: parsed.options.timeoutMs || undefined,
		headless: parsed.options.headless || undefined,
		region: parsed.options.region || undefined,
		solveCaptcha: parsed.options.solveCaptcha || undefined,
		deadSessionBehavior: 'error',
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
