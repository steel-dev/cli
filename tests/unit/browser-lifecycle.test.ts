import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	type Mock,
	vi,
	describe,
	test,
	expect,
	beforeEach,
	afterEach,
} from 'vitest';

type BrowserLifecycleModule =
	typeof import('../../source/utils/browser/lifecycle');
type SessionState = {
	activeSessionId: string | null;
	activeSessionMode: 'cloud' | 'local' | null;
	activeSessionName: string | null;
	namedSessions: {
		cloud: Record<string, string>;
		local: Record<string, string>;
	};
	updatedAt: string | null;
};

function createJsonResponse(status: number, payload: unknown): Response {
	return {
		ok: status >= 200 && status < 300,
		status,
		statusText: status >= 200 && status < 300 ? 'OK' : 'Error',
		text: async () => JSON.stringify(payload),
	} as Response;
}

function createTempConfigDirectory(): string {
	return fs.mkdtempSync(
		path.join(os.tmpdir(), 'steel-browser-lifecycle-test-'),
	);
}

function readSessionState(configDirectory: string): SessionState {
	const statePath = path.join(configDirectory, 'browser-session-state.json');
	const raw = fs.readFileSync(statePath, 'utf-8');
	return JSON.parse(raw) as SessionState;
}

function writeSessionState(configDirectory: string, state: SessionState): void {
	const statePath = path.join(configDirectory, 'browser-session-state.json');
	fs.mkdirSync(configDirectory, {recursive: true});
	fs.writeFileSync(statePath, JSON.stringify(state, null, 2), 'utf-8');
}

async function loadBrowserLifecycle(
	configDirectory: string,
): Promise<BrowserLifecycleModule> {
	process.env.STEEL_CONFIG_DIR = configDirectory;
	vi.resetModules();
	return import('../../source/utils/browser/lifecycle');
}

const originalFetch = globalThis.fetch;
let fetchMock: Mock<typeof fetch>;

beforeEach(() => {
	fetchMock = vi.fn() as Mock<typeof fetch>;
	globalThis.fetch = fetchMock;
});

afterEach(() => {
	globalThis.fetch = originalFetch;
	delete process.env.STEEL_CONFIG_DIR;
});

describe('browser lifecycle passthrough bootstrap parsing', () => {
	test('extracts bootstrap options and keeps runtime args', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const parsed = lifecycle.parseBrowserPassthroughBootstrapFlags([
				'open',
				'https://steel.dev',
				'--session',
				'daily',
				'--stealth',
				'--proxy',
				'http://proxy.local:8080',
				'--wait-for',
				'load',
			]);

			expect(parsed.options).toEqual({
				local: false,
				sessionName: 'daily',
				stealth: true,
				proxyUrl: 'http://proxy.local:8080',
				autoConnect: false,
				cdpTarget: null,
			});
			expect(parsed.passthroughArgv).toEqual([
				'open',
				'https://steel.dev',
				'--wait-for',
				'load',
			]);
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('throws for invalid bootstrap flag values', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);

			expect(() =>
				lifecycle.parseBrowserPassthroughBootstrapFlags(['open', '--session']),
			).toThrow('Missing value for --session.');
			expect(() =>
				lifecycle.parseBrowserPassthroughBootstrapFlags(['open', '--proxy']),
			).toThrow('Missing value for --proxy.');
			expect(() =>
				lifecycle.parseBrowserPassthroughBootstrapFlags(['open', '--cdp']),
			).toThrow('Missing value for --cdp.');
			expect(() =>
				lifecycle.parseBrowserPassthroughBootstrapFlags([
					'open',
					'--auto-connect',
					'ws://localhost:9222',
				]),
			).toThrow('`--auto-connect` does not accept a value.');
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('keeps explicit auto-connect untouched', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const result = await lifecycle.bootstrapBrowserPassthroughArgv(
				['open', 'https://steel.dev', '--auto-connect'],
				{STEEL_API_KEY: 'env-api-key'},
			);

			expect(result).toEqual(['open', 'https://steel.dev', '--auto-connect']);
			expect(fetchMock).not.toHaveBeenCalled();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('keeps explicit cdp target untouched', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const result = await lifecycle.bootstrapBrowserPassthroughArgv(
				['open', 'https://steel.dev', '--cdp', 'ws://localhost:9222'],
				{STEEL_API_KEY: 'env-api-key'},
			);

			expect(result).toEqual([
				'open',
				'https://steel.dev',
				'--cdp',
				'ws://localhost:9222',
			]);
			expect(fetchMock).not.toHaveBeenCalled();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('rejects mixed explicit attach flags', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);

			expect(() =>
				lifecycle.parseBrowserPassthroughBootstrapFlags([
					'open',
					'https://steel.dev',
					'--auto-connect',
					'--cdp',
					'ws://localhost:9222',
				]),
			).toThrow('Cannot combine `--auto-connect` with `--cdp`.');
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('skips bootstrap for help output', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const result = await lifecycle.bootstrapBrowserPassthroughArgv(
				['open', '--help'],
				{STEEL_API_KEY: 'env-api-key'},
			);

			expect(result).toEqual(['open', '--help']);
			expect(fetchMock).not.toHaveBeenCalled();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});
});

describe('browser lifecycle session contract', () => {
	test('creates cloud session and persists active state', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock.mockResolvedValueOnce(
				createJsonResponse(201, {
					id: 'session-created',
					status: 'live',
					websocketUrl: 'wss://connect.steel.dev/session-created',
					viewerUrl: 'https://app.steel.dev/sessions/session-created',
				}),
			);

			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const session = await lifecycle.startBrowserSession({
				environment: {STEEL_API_KEY: 'env-api-key'},
			});

			expect(session).toMatchObject({
				id: 'session-created',
				mode: 'cloud',
				live: true,
				connectUrl:
					'wss://connect.steel.dev/session-created?apiKey=env-api-key',
				viewerUrl: 'https://app.steel.dev/sessions/session-created',
			});

			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions',
			);
			expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Steel-Api-Key': 'env-api-key',
				},
				body: '{}',
			});

			const state = readSessionState(configDirectory);
			expect(state.activeSessionMode).toBe('cloud');
			expect(state.activeSessionId).toBe('session-created');
			expect(state.activeSessionName).toBeNull();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('reattaches a named live session instead of creating a new one', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock
				.mockResolvedValueOnce(
					createJsonResponse(201, {
						id: 'session-daily',
						status: 'live',
						websocketUrl: 'wss://connect.steel.dev/session-daily',
					}),
				)
				.mockResolvedValueOnce(
					createJsonResponse(200, {
						id: 'session-daily',
						status: 'live',
						websocketUrl: 'wss://connect.steel.dev/session-daily',
					}),
				);

			const lifecycle = await loadBrowserLifecycle(configDirectory);

			const firstSession = await lifecycle.startBrowserSession({
				sessionName: 'daily',
				environment: {STEEL_API_KEY: 'env-api-key'},
			});
			const secondSession = await lifecycle.startBrowserSession({
				sessionName: 'daily',
				environment: {STEEL_API_KEY: 'env-api-key'},
			});

			expect(firstSession.id).toBe('session-daily');
			expect(secondSession.id).toBe('session-daily');

			expect(fetchMock).toHaveBeenCalledTimes(2);
			expect(fetchMock.mock.calls[0]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions',
			);
			expect(fetchMock.mock.calls[0]?.[1]?.method).toBe('POST');
			expect(fetchMock.mock.calls[1]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions/session-daily',
			);
			expect(fetchMock.mock.calls[1]?.[1]?.method).toBe('GET');

			const state = readSessionState(configDirectory);
			expect(state.namedSessions.cloud).toEqual({
				daily: 'session-daily',
			});
			expect(state.activeSessionId).toBe('session-daily');
			expect(state.activeSessionName).toBe('daily');
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('injects cdp URL with create-on-first-action bootstrap', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock.mockResolvedValueOnce(
				createJsonResponse(201, {
					id: 'session-bootstrap',
					status: 'live',
				}),
			);

			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const passthroughArgv = await lifecycle.bootstrapBrowserPassthroughArgv(
				[
					'open',
					'https://steel.dev',
					'--session',
					'daily',
					'--stealth',
					'--proxy',
					'http://proxy.local:8080',
				],
				{
					STEEL_API_KEY: 'env-api-key',
				},
			);

			expect(passthroughArgv).toEqual([
				'open',
				'https://steel.dev',
				'--cdp',
				'wss://connect.steel.dev?apiKey=env-api-key&sessionId=session-bootstrap',
			]);

			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions',
			);
			expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
				method: 'POST',
				body: JSON.stringify({
					proxyUrl: 'http://proxy.local:8080',
					stealthConfig: {
						humanizeInteractions: true,
						autoCaptchaSolving: true,
					},
					solveCaptchas: true,
				}),
			});
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('returns missing-auth error before API calls when no auth is present', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			const lifecycle = await loadBrowserLifecycle(configDirectory);

			await expect(
				lifecycle.startBrowserSession({
					environment: {
						STEEL_CONFIG_DIR: configDirectory,
					},
				}),
			).rejects.toMatchObject({
				code: 'MISSING_AUTH',
			});

			expect(fetchMock).not.toHaveBeenCalled();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('stops all live sessions and clears active cloud state', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			writeSessionState(configDirectory, {
				activeSessionId: 'session-a',
				activeSessionMode: 'cloud',
				activeSessionName: 'daily',
				namedSessions: {
					cloud: {
						daily: 'session-a',
						old: 'session-z',
					},
					local: {},
				},
				updatedAt: null,
			});

			fetchMock
				.mockResolvedValueOnce(
					createJsonResponse(200, {
						sessions: [
							{id: 'session-a', status: 'live'},
							{id: 'session-b', status: 'terminated'},
							{id: 'session-c', status: 'active'},
						],
					}),
				)
				.mockResolvedValueOnce(createJsonResponse(200, {ok: true}))
				.mockResolvedValueOnce(createJsonResponse(200, {ok: true}));

			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const result = await lifecycle.stopBrowserSession({
				all: true,
				environment: {STEEL_API_KEY: 'env-api-key'},
			});

			expect(result).toEqual({
				mode: 'cloud',
				all: true,
				stoppedSessionIds: ['session-a', 'session-c'],
			});
			expect(fetchMock).toHaveBeenCalledTimes(3);
			expect(fetchMock.mock.calls[1]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions/session-a/release',
			);
			expect(fetchMock.mock.calls[2]?.[0]).toBe(
				'https://api.steel.dev/v1/sessions/session-c/release',
			);

			const state = readSessionState(configDirectory);
			expect(state.activeSessionMode).toBeNull();
			expect(state.activeSessionId).toBeNull();
			expect(state.namedSessions.cloud).toEqual({
				old: 'session-z',
			});
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('returns fallback cloud live URL for the active session', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			writeSessionState(configDirectory, {
				activeSessionId: 'session-live',
				activeSessionMode: 'cloud',
				activeSessionName: null,
				namedSessions: {
					cloud: {},
					local: {},
				},
				updatedAt: null,
			});

			fetchMock.mockResolvedValueOnce(
				createJsonResponse(200, {
					id: 'session-live',
					status: 'live',
				}),
			);

			const lifecycle = await loadBrowserLifecycle(configDirectory);
			const liveUrl = await lifecycle.getActiveBrowserLiveUrl({
				environment: {STEEL_API_KEY: 'env-api-key'},
			});

			expect(liveUrl).toBe('https://app.steel.dev/sessions/session-live');
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});
});
