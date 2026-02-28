export type BrowserSessionMode = 'cloud' | 'local';
export type DeadSessionBehavior = 'recreate' | 'error';

export type UnknownRecord = Record<string, unknown>;

export type BrowserSessionState = {
	activeSessionId: string | null;
	activeSessionMode: BrowserSessionMode | null;
	activeSessionName: string | null;
	namedSessions: {
		cloud: Record<string, string>;
		local: Record<string, string>;
	};
	updatedAt: string | null;
};

export type StartSessionRequestOptions = {
	stealth?: boolean;
	proxyUrl?: string;
	timeoutMs?: number;
	headless?: boolean;
	region?: string;
	solveCaptcha?: boolean;
};

export type SolveCaptchaRequestOptions = {
	pageId?: string;
	url?: string;
	taskId?: string;
};

export type ParsedBootstrapOptions = {
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
	sessionName?: string;
	local?: boolean;
	apiUrl?: string;
	environment?: NodeJS.ProcessEnv;
};

export type SolveBrowserSessionCaptchaOptions = {
	sessionId?: string;
	sessionName?: string;
	local?: boolean;
	apiUrl?: string;
	pageId?: string;
	url?: string;
	taskId?: string;
	environment?: NodeJS.ProcessEnv;
};

export type BrowserSessionEndpointOptions = {
	sessionName?: string;
	local?: boolean;
	apiUrl?: string;
	environment?: NodeJS.ProcessEnv;
};

export type StopBrowserSessionResult = {
	mode: BrowserSessionMode;
	all: boolean;
	stoppedSessionIds: string[];
};

export type SolveBrowserSessionCaptchaResult = {
	mode: BrowserSessionMode;
	sessionId: string;
	success: boolean;
	message: string | null;
	raw: UnknownRecord;
};

export type GetCaptchaStatusRequestOptions = {
	pageId?: string;
};

export type GetBrowserSessionCaptchaStatusOptions = {
	sessionId?: string;
	sessionName?: string;
	local?: boolean;
	apiUrl?: string;
	pageId?: string;
	wait?: boolean;
	timeout?: number;
	interval?: number;
	environment?: NodeJS.ProcessEnv;
};

export type CaptchaStatusValue = 'none' | 'solving' | 'solved' | 'failed';

export type CaptchaType =
	| 'recaptchaV2'
	| 'recaptchaV3'
	| 'turnstile'
	| 'image_to_text';

export type GetBrowserSessionCaptchaStatusResult = {
	mode: BrowserSessionMode;
	sessionId: string;
	status: CaptchaStatusValue;
	types: CaptchaType[];
	raw: UnknownRecord;
};
