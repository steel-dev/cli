import * as fsSync from 'node:fs';
import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {getLocalBrowserRepoPath} from '../../dev/local.js';
import {resolveBrowserAuth} from '../auth.js';
import {BrowserAdapterError} from '../errors.js';
import {DEFAULT_API_PATH, DEFAULT_LOCAL_API_PATH} from './constants.js';
import type {
	BrowserSessionMode,
	GetCaptchaStatusRequestOptions,
	SolveCaptchaRequestOptions,
	StartSessionRequestOptions,
	UnknownRecord,
} from './types.js';

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

export function resolveExplicitApiUrl(apiUrl?: string | null): string | null {
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

export function parsePositiveIntegerFlagValue(
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

export async function listSessionsFromApi(
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

export async function getSessionFromApi(
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

export async function createSessionFromApi(
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

export async function solveSessionCaptchaFromApi(
	mode: BrowserSessionMode,
	sessionId: string,
	options: SolveCaptchaRequestOptions,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord> {
	const payload: UnknownRecord = {};

	if (options.pageId?.trim()) {
		payload['pageId'] = options.pageId.trim();
	}

	if (options.url?.trim()) {
		payload['url'] = options.url.trim();
	}

	if (options.taskId?.trim()) {
		payload['taskId'] = options.taskId.trim();
	}

	const responseData = await requestApi(
		mode,
		environment,
		`/sessions/${sessionId}/captchas/solve`,
		'POST',
		payload,
		apiUrl,
	);

	if (!responseData || typeof responseData !== 'object') {
		throw new BrowserAdapterError(
			'API_ERROR',
			'Unexpected empty response from Steel captcha solve endpoint.',
		);
	}

	return responseData as UnknownRecord;
}

export async function getCaptchaStatusFromApi(
	mode: BrowserSessionMode,
	sessionId: string,
	options: GetCaptchaStatusRequestOptions,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<UnknownRecord[]> {
	let endpoint = `/sessions/${sessionId}/captchas/status`;
	if (options.pageId?.trim()) {
		endpoint += `?pageId=${encodeURIComponent(options.pageId.trim())}`;
	}

	const responseData = await requestApi(
		mode,
		environment,
		endpoint,
		'GET',
		undefined,
		apiUrl,
	);

	if (!responseData) {
		return [];
	}

	if (Array.isArray(responseData)) {
		return responseData.filter(
			item => item && typeof item === 'object',
		) as UnknownRecord[];
	}

	if (typeof responseData === 'object') {
		return [responseData as UnknownRecord];
	}

	return [];
}

export async function releaseSession(
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
