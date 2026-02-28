import {BrowserAdapterError} from './errors.js';
import {
	createSessionFromApi,
	getCaptchaStatusFromApi,
	getSessionFromApi,
	listSessionsFromApi,
	releaseSession,
	resolveExplicitApiUrl,
	solveSessionCaptchaFromApi,
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
	CaptchaStatusValue,
	CaptchaType,
	GetBrowserSessionCaptchaStatusOptions,
	GetBrowserSessionCaptchaStatusResult,
	SolveBrowserSessionCaptchaOptions,
	SolveBrowserSessionCaptchaResult,
	StartBrowserSessionOptions,
	StopBrowserSessionOptions,
	StopBrowserSessionResult,
	UnknownRecord,
} from './lifecycle/types.js';

export {parseBrowserPassthroughBootstrapFlags};
export type {
	BrowserSessionMode,
	BrowserSessionSummary,
	CaptchaStatusValue,
	CaptchaType,
	GetBrowserSessionCaptchaStatusOptions,
	GetBrowserSessionCaptchaStatusResult,
	SolveBrowserSessionCaptchaOptions,
	SolveBrowserSessionCaptchaResult,
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

function resolveTargetSession(
	options: {sessionName?: string},
	mode: BrowserSessionMode,
	state: Awaited<ReturnType<typeof readSessionState>>,
): {sessionId: string | null; sessionName: string | null} {
	const sessionName = options.sessionName?.trim();
	if (sessionName) {
		return {
			sessionId: state.namedSessions[mode][sessionName] || null,
			sessionName,
		};
	}

	if (state.activeSessionMode === mode && state.activeSessionId) {
		return {
			sessionId: state.activeSessionId,
			sessionName: state.activeSessionName,
		};
	}

	return {
		sessionId: null,
		sessionName: null,
	};
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

	const targetSession = resolveTargetSession(
		{sessionName: options?.sessionName},
		mode,
		state,
	);
	if (!targetSession.sessionId) {
		return null;
	}

	const session = await tryGetLiveSession(
		mode,
		targetSession.sessionId,
		environment,
		apiUrl,
	);
	if (!session) {
		return null;
	}

	const summary = toSessionSummary(
		session,
		mode,
		targetSession.sessionName,
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
				targetSessionId: resolveTargetSession(
					{sessionName: options.sessionName},
					mode,
					state,
				).sessionId,
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

function resolveSolveCaptchaSessionId(
	options: SolveBrowserSessionCaptchaOptions,
	mode: BrowserSessionMode,
	state: Awaited<ReturnType<typeof readSessionState>>,
): string | null {
	const explicitSessionId = options.sessionId?.trim();
	if (explicitSessionId) {
		return explicitSessionId;
	}

	const sessionName = options.sessionName?.trim();
	if (sessionName) {
		return state.namedSessions[mode][sessionName] || null;
	}

	if (state.activeSessionMode === mode && state.activeSessionId) {
		return state.activeSessionId;
	}

	return null;
}

export async function solveBrowserSessionCaptcha(
	options: SolveBrowserSessionCaptchaOptions = {},
): Promise<SolveBrowserSessionCaptchaResult> {
	const environment = options.environment || process.env;
	const apiUrl = resolveExplicitApiUrl(options.apiUrl);
	const state = await readSessionState();
	const mode = resolveSessionMode(
		options.local,
		apiUrl,
		state.activeSessionMode,
		environment,
	);
	const sessionId = resolveSolveCaptchaSessionId(options, mode, state);

	if (!sessionId) {
		throw new BrowserAdapterError(
			'SESSION_NOT_FOUND',
			'No target browser session found for CAPTCHA solving. Pass `--session-id`, pass `--session <name>`, or start a session first with `steel browser start --session <name>`.',
		);
	}

	const rawResponse = await solveSessionCaptchaFromApi(
		mode,
		sessionId,
		{
			pageId: options.pageId,
			url: options.url,
			taskId: options.taskId,
		},
		environment,
		apiUrl,
	);
	const success = rawResponse['success'];
	const message = rawResponse['message'];

	return {
		mode,
		sessionId,
		success: typeof success === 'boolean' ? success : false,
		message: typeof message === 'string' ? message : null,
		raw: rawResponse,
	};
}

const DEFAULT_CAPTCHA_WAIT_TIMEOUT_MS = 60_000;
const DEFAULT_CAPTCHA_POLL_INTERVAL_MS = 1_000;

const KNOWN_CAPTCHA_TYPES: CaptchaType[] = [
	'recaptchaV2',
	'recaptchaV3',
	'turnstile',
	'image_to_text',
];

function wait(milliseconds: number): Promise<void> {
	return new Promise(resolve => {
		setTimeout(resolve, milliseconds);
	});
}

function normalizeCaptchaStatus(pages: UnknownRecord[]): {
	status: CaptchaStatusValue;
	types: CaptchaType[];
} {
	if (!pages.length) {
		return {status: 'none', types: []};
	}

	const allTasks: UnknownRecord[] = [];
	let anySolving = false;

	for (const page of pages) {
		if (page['isSolvingCaptcha'] === true) {
			anySolving = true;
		}

		const tasks = page['tasks'];
		if (Array.isArray(tasks)) {
			for (const task of tasks) {
				if (task && typeof task === 'object') {
					allTasks.push(task as UnknownRecord);
				}
			}
		}
	}

	if (!allTasks.length) {
		return {status: anySolving ? 'solving' : 'none', types: []};
	}

	const solvingTypes: Set<CaptchaType> = new Set();
	const failedTypes: Set<CaptchaType> = new Set();
	let hasSolving = false;
	let hasFailed = false;
	let hasSolved = false;

	for (const task of allTasks) {
		const taskStatus = task['status'];
		const taskType = task['type'];
		const captchaType = KNOWN_CAPTCHA_TYPES.includes(taskType as CaptchaType)
			? (taskType as CaptchaType)
			: null;

		if (
			taskStatus === 'solving' ||
			taskStatus === 'detected' ||
			taskStatus === 'validating'
		) {
			hasSolving = true;
			if (captchaType) {
				solvingTypes.add(captchaType);
			}
		} else if (
			taskStatus === 'failed_to_solve' ||
			taskStatus === 'failed_to_detect' ||
			taskStatus === 'validation_failed'
		) {
			hasFailed = true;
			if (captchaType) {
				failedTypes.add(captchaType);
			}
		} else if (taskStatus === 'solved') {
			hasSolved = true;
		}
	}

	if (hasSolving || anySolving) {
		return {status: 'solving', types: Array.from(solvingTypes)};
	}

	if (hasFailed) {
		return {status: 'failed', types: Array.from(failedTypes)};
	}

	if (hasSolved) {
		return {status: 'solved', types: []};
	}

	return {status: 'none', types: []};
}

function isTerminalCaptchaStatus(status: CaptchaStatusValue): boolean {
	return status === 'solved' || status === 'failed' || status === 'none';
}

export async function getBrowserSessionCaptchaStatus(
	options: GetBrowserSessionCaptchaStatusOptions = {},
): Promise<GetBrowserSessionCaptchaStatusResult> {
	const environment = options.environment || process.env;
	const apiUrl = resolveExplicitApiUrl(options.apiUrl);
	const state = await readSessionState();
	const mode = resolveSessionMode(
		options.local,
		apiUrl,
		state.activeSessionMode,
		environment,
	);
	const sessionId = resolveSolveCaptchaSessionId(options, mode, state);

	if (!sessionId) {
		throw new BrowserAdapterError(
			'SESSION_NOT_FOUND',
			'No target browser session found for CAPTCHA status. Pass `--session-id`, pass `--session <name>`, or start a session first with `steel browser start --session <name>`.',
		);
	}

	const timeout = options.timeout ?? DEFAULT_CAPTCHA_WAIT_TIMEOUT_MS;
	const interval = options.interval ?? DEFAULT_CAPTCHA_POLL_INTERVAL_MS;
	const startTime = Date.now();

	while (true) {
		const pages = await getCaptchaStatusFromApi(
			mode,
			sessionId,
			{pageId: options.pageId},
			environment,
			apiUrl,
		);

		const {status, types} = normalizeCaptchaStatus(pages);

		if (!options.wait || isTerminalCaptchaStatus(status)) {
			return {
				mode,
				sessionId,
				status,
				types,
				raw: {pages},
			};
		}

		if (Date.now() - startTime >= timeout) {
			throw new BrowserAdapterError(
				'API_ERROR',
				`CAPTCHA status polling timed out after ${timeout}ms. Last status: ${status}`,
			);
		}

		await wait(interval);
	}
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
