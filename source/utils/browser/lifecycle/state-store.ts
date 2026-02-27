import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {BrowserAdapterError} from '../errors.js';
import type {BrowserSessionMode, BrowserSessionState} from './types.js';

const CONFIG_DIR =
	process.env.STEEL_CONFIG_DIR?.trim() ||
	path.join(os.homedir(), '.config', 'steel');
const SESSION_STATE_PATH = path.join(CONFIG_DIR, 'browser-session-state.json');
const SESSION_STATE_LOCK_PATH = `${SESSION_STATE_PATH}.lock`;
const LOCK_TIMEOUT_MS = 5000;
const LOCK_RETRY_MS = 50;
const LOCK_STALE_MS = 15_000;

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

export async function readSessionState(): Promise<BrowserSessionState> {
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

export async function withSessionStateLock<T>(
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

export function setActiveSessionState(
	state: BrowserSessionState,
	mode: BrowserSessionMode,
	sessionId: string,
	sessionName: string | null,
): void {
	state.activeSessionMode = mode;
	state.activeSessionId = sessionId;
	state.activeSessionName = sessionName;
}

export function clearActiveSessionState(
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

export function resolveCandidateSessionId(
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

export function resolveNameFromState(
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
