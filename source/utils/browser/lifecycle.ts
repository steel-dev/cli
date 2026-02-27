import {BrowserAdapterError} from './errors.js';
import {
	createSessionFromApi,
	getSessionFromApi,
	listSessionsFromApi,
	releaseSession,
	resolveExplicitApiUrl,
} from './lifecycle/api-client.js';
import {parseBrowserPassthroughBootstrapFlags} from './lifecycle/bootstrap-flags.js';
import {
	buildDeadSessionMessage,
	getSessionId,
	isNotFoundApiError,
	isSessionLive,
	resolveSessionMode,
	toSessionSummary,
} from './lifecycle/session-policy.js';
import {
	clearActiveSessionState,
	readSessionState,
	resolveCandidateSessionId,
	resolveNameFromState,
	setActiveSessionState,
	withSessionStateLock,
} from './lifecycle/state-store.js';
import type {
	BrowserSessionEndpointOptions,
	BrowserSessionMode,
	BrowserSessionSummary,
	StartBrowserSessionOptions,
	StopBrowserSessionOptions,
	StopBrowserSessionResult,
	UnknownRecord,
} from './lifecycle/types.js';

export {parseBrowserPassthroughBootstrapFlags};
export type {
	BrowserSessionMode,
	BrowserSessionSummary,
	StartBrowserSessionOptions,
	StopBrowserSessionOptions,
	StopBrowserSessionResult,
};

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
