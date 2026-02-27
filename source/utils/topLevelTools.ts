import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {API_PATH, LOCAL_API_PATH} from './constants.js';
import {resolveBrowserAuth} from './browser/auth.js';
import {BrowserAdapterError} from './browser/errors.js';

type UnknownRecord = Record<string, unknown>;

export type TopLevelApiMode = 'cloud' | 'local';

export type TopLevelApiRequestOptions = {
	local?: boolean;
	apiUrl?: string;
	environment?: NodeJS.ProcessEnv;
};

const SCRAPE_FORMAT_VALUES = [
	'html',
	'readability',
	'cleaned_html',
	'markdown',
] as const;

const SCRAPE_FORMAT_SET = new Set<string>(SCRAPE_FORMAT_VALUES);

export type ScrapeFormat = (typeof SCRAPE_FORMAT_VALUES)[number];
type ScrapeContentKey = 'markdown' | 'cleaned_html' | 'html' | 'readability';

function normalizeApiBaseUrl(url: string): string {
	return url.replace(/\/+$/, '');
}

function hasExplicitNavigationProtocol(url: string): boolean {
	const normalized = url.toLowerCase();
	return (
		normalized.includes('://') ||
		normalized.startsWith('about:') ||
		normalized.startsWith('data:') ||
		normalized.startsWith('file:') ||
		normalized.startsWith('blob:') ||
		normalized.startsWith('javascript:')
	);
}

function looksLikeHostWithoutProtocol(url: string): boolean {
	const hostCandidate = url.split('/')[0] || '';
	const normalizedHostCandidate = hostCandidate.toLowerCase();

	return (
		normalizedHostCandidate === 'localhost' ||
		normalizedHostCandidate.startsWith('localhost:') ||
		(normalizedHostCandidate.startsWith('[') &&
			normalizedHostCandidate.includes(']')) ||
		/^\d{1,3}(?:\.\d{1,3}){3}(?::\d+)?$/.test(normalizedHostCandidate) ||
		hostCandidate.includes('.')
	);
}

function normalizeUrlWithHttpsFallback(url: string): string {
	const trimmedUrl = url.trim();
	if (
		!trimmedUrl ||
		hasExplicitNavigationProtocol(trimmedUrl) ||
		!looksLikeHostWithoutProtocol(trimmedUrl)
	) {
		return trimmedUrl;
	}

	const normalizedUrl = `https://${trimmedUrl}`;
	try {
		new URL(normalizedUrl);
		return normalizedUrl;
	} catch {
		return trimmedUrl;
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

	try {
		return normalizeApiBaseUrl(new URL(trimmedApiUrl).toString());
	} catch {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid value for --api-url: ${apiUrl}`,
		);
	}
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

async function getApiBaseUrl(
	mode: TopLevelApiMode,
	environment: NodeJS.ProcessEnv,
	apiUrl?: string | null,
): Promise<string> {
	if (mode === 'local') {
		const explicitApiUrl = resolveExplicitApiUrl(apiUrl);
		if (explicitApiUrl) {
			return explicitApiUrl;
		}

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

		return normalizeApiBaseUrl(LOCAL_API_PATH);
	}

	return normalizeApiBaseUrl(environment.STEEL_API_URL?.trim() || API_PATH);
}

function resolveMode(
	local: boolean | undefined,
	apiUrl: string | null,
): TopLevelApiMode {
	return local || apiUrl ? 'local' : 'cloud';
}

function extractApiErrorMessage(
	responseData: unknown,
	statusText: string,
): string {
	if (typeof responseData === 'string' && responseData.trim()) {
		return responseData;
	}

	if (responseData && typeof responseData === 'object') {
		const payload = responseData as UnknownRecord;
		if (typeof payload['message'] === 'string' && payload['message'].trim()) {
			return payload['message'];
		}

		const nestedError = payload['error'];
		if (nestedError && typeof nestedError === 'object') {
			const nestedMessage = (nestedError as UnknownRecord)['message'];
			if (typeof nestedMessage === 'string' && nestedMessage.trim()) {
				return nestedMessage;
			}
		}
	}

	return statusText || 'Unknown API error';
}

export function parseScrapeFormatOption(
	formatOption?: string,
): ScrapeFormat[] | undefined {
	if (formatOption === undefined) {
		return undefined;
	}

	const formats = formatOption
		.split(',')
		.map(part => part.trim())
		.filter(Boolean);

	if (formats.length === 0) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'Missing value for --format. Example: --format html,markdown',
		);
	}

	const invalidFormats = formats.filter(value => !SCRAPE_FORMAT_SET.has(value));
	if (invalidFormats.length > 0) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid scrape format value(s): ${invalidFormats.join(', ')}. Supported values: ${SCRAPE_FORMAT_VALUES.join(', ')}`,
		);
	}

	return formats as ScrapeFormat[];
}

function pushUniqueContentKey(
	keys: ScrapeContentKey[],
	key: ScrapeContentKey,
): void {
	if (!keys.includes(key)) {
		keys.push(key);
	}
}

export function getScrapeOutputText(
	responseData: unknown,
	preferredFormats: ScrapeFormat[] = ['markdown'],
): string | null {
	if (!responseData || typeof responseData !== 'object') {
		return null;
	}

	const content = (responseData as UnknownRecord)['content'];
	if (!content || typeof content !== 'object') {
		return null;
	}

	const contentRecord = content as UnknownRecord;
	const orderedKeys: ScrapeContentKey[] = [];

	for (const format of preferredFormats) {
		pushUniqueContentKey(orderedKeys, format);
	}

	pushUniqueContentKey(orderedKeys, 'markdown');
	pushUniqueContentKey(orderedKeys, 'cleaned_html');
	pushUniqueContentKey(orderedKeys, 'html');
	pushUniqueContentKey(orderedKeys, 'readability');

	for (const key of orderedKeys) {
		const value = contentRecord[key];
		if (typeof value === 'string' && value.trim()) {
			return value;
		}

		if (key === 'readability' && value && typeof value === 'object') {
			return JSON.stringify(value, null, 2);
		}
	}

	return null;
}

export function resolveTopLevelToolUrl(
	urlFromOption?: string,
	urlFromArg?: string,
): string {
	const candidate = (urlFromOption || urlFromArg || '').trim();
	if (!candidate) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'Missing URL. Provide a target URL as the first argument or with --url.',
		);
	}

	const normalizedCandidate = normalizeUrlWithHttpsFallback(candidate);

	try {
		new URL(normalizedCandidate);
	} catch {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid URL: ${candidate}`,
		);
	}

	return normalizedCandidate;
}

export function getHostedAssetUrl(responseData: unknown): string {
	if (responseData && typeof responseData === 'object') {
		const url = (responseData as UnknownRecord)['url'];
		if (typeof url === 'string' && url.trim()) {
			return url.trim();
		}
	}

	throw new BrowserAdapterError(
		'API_ERROR',
		'Steel API response did not include a URL.',
	);
}

export async function requestTopLevelApi<TResponse>(
	endpointPath: '/scrape' | '/screenshot' | '/pdf',
	body: UnknownRecord,
	options: TopLevelApiRequestOptions = {},
): Promise<TResponse> {
	const environment = options.environment || process.env;
	const explicitApiUrl = resolveExplicitApiUrl(options.apiUrl);
	const mode = resolveMode(options.local, explicitApiUrl);
	const auth = resolveBrowserAuth(environment);

	if (mode === 'cloud' && !auth.apiKey) {
		throw new BrowserAdapterError(
			'MISSING_AUTH',
			'Missing Steel API key. Run `steel login` or set `STEEL_API_KEY`.',
		);
	}

	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
	};

	if (auth.apiKey) {
		headers['Steel-Api-Key'] = auth.apiKey;
	}

	const apiBaseUrl = await getApiBaseUrl(mode, environment, explicitApiUrl);
	let response: Response;

	try {
		response = await fetch(`${apiBaseUrl}${endpointPath}`, {
			method: 'POST',
			headers,
			body: JSON.stringify(body),
		});
	} catch (error) {
		throw new BrowserAdapterError(
			'API_ERROR',
			`Failed to reach Steel API at ${apiBaseUrl}.`,
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
		throw new BrowserAdapterError(
			'API_ERROR',
			`Steel API request failed (${response.status}): ${extractApiErrorMessage(responseData, response.statusText)}`,
		);
	}

	return responseData as TResponse;
}
