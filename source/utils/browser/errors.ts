export type BrowserAdapterErrorCode =
	| 'MISSING_AUTH'
	| 'RUNTIME_NOT_FOUND'
	| 'SPAWN_ERROR'
	| 'INVALID_BROWSER_ARGS'
	| 'API_ERROR'
	| 'SESSION_NOT_FOUND';

export class BrowserAdapterError extends Error {
	readonly code: BrowserAdapterErrorCode;
	readonly cause?: unknown;

	constructor(code: BrowserAdapterErrorCode, message: string, cause?: unknown) {
		super(message);
		this.name = 'BrowserAdapterError';
		this.code = code;
		this.cause = cause;
	}
}
