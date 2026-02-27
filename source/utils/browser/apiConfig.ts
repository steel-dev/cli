import * as fsSync from 'node:fs';
import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {API_PATH, LOCAL_API_PATH} from '../constants.js';
import {BrowserAdapterError} from './errors.js';

type UnknownRecord = Record<string, unknown>;

export type BrowserApiMode = 'cloud' | 'local';

export function normalizeApiBaseUrl(url: string): string {
	return url.replace(/\/+$/, '');
}

export function getConfigDirectory(environment: NodeJS.ProcessEnv): string {
	return (
		environment.STEEL_CONFIG_DIR?.trim() ||
		path.join(os.homedir(), '.config', 'steel')
	);
}

export function getConfigPath(environment: NodeJS.ProcessEnv): string {
	return path.join(getConfigDirectory(environment), 'config.json');
}

function coerceTrimmedString(value: unknown): string | null {
	if (typeof value === 'string' && value.trim()) {
		return value.trim();
	}

	return null;
}

function coerceLocalApiUrlFromConfig(config: unknown): string | null {
	if (!config || typeof config !== 'object') {
		return null;
	}

	const browser = (config as UnknownRecord)['browser'];
	if (!browser || typeof browser !== 'object') {
		return null;
	}

	return coerceTrimmedString((browser as UnknownRecord)['apiUrl']);
}

function coerceApiKeyFromConfig(config: unknown): string | null {
	if (!config || typeof config !== 'object') {
		return null;
	}

	return coerceTrimmedString((config as UnknownRecord)['apiKey']);
}

function readConfigSync(environment: NodeJS.ProcessEnv): unknown | null {
	try {
		const configPath = getConfigPath(environment);
		const configContents = fsSync.readFileSync(configPath, 'utf-8');
		return JSON.parse(configContents) as unknown;
	} catch {
		return null;
	}
}

async function readConfig(
	environment: NodeJS.ProcessEnv,
): Promise<unknown | null> {
	try {
		const configPath = getConfigPath(environment);
		const configContents = await fs.readFile(configPath, 'utf-8');
		return JSON.parse(configContents) as unknown;
	} catch {
		return null;
	}
}

export function readApiKeyFromConfig(
	environment: NodeJS.ProcessEnv,
): string | null {
	const config = readConfigSync(environment);
	return coerceApiKeyFromConfig(config);
}

export async function readLocalApiUrlFromConfig(
	environment: NodeJS.ProcessEnv,
): Promise<string | null> {
	const config = await readConfig(environment);
	return coerceLocalApiUrlFromConfig(config);
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

	try {
		return normalizeApiBaseUrl(new URL(trimmedApiUrl).toString());
	} catch {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			`Invalid value for --api-url: ${apiUrl}`,
		);
	}
}

export function resolveCloudApiBaseUrl(environment: NodeJS.ProcessEnv): string {
	return normalizeApiBaseUrl(environment.STEEL_API_URL?.trim() || API_PATH);
}

export async function resolveApiBaseUrl(
	mode: BrowserApiMode,
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

		const configApiUrl = await readLocalApiUrlFromConfig(environment);
		if (configApiUrl) {
			return normalizeApiBaseUrl(configApiUrl);
		}

		return normalizeApiBaseUrl(LOCAL_API_PATH);
	}

	return resolveCloudApiBaseUrl(environment);
}
