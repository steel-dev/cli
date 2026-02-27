import {describe, test, expect} from 'vitest';
import {sanitizeConnectUrlForDisplay} from '../../source/utils/browser/display';

describe('browser start connect url display sanitization', () => {
	test('returns connect URL unchanged when no sensitive query keys exist', () => {
		const connectUrl =
			'wss://connect.steel.dev/session-123?sessionId=session-123&region=us-west-1';

		expect(sanitizeConnectUrlForDisplay(connectUrl)).toBe(connectUrl);
	});

	test('redacts api key style query parameters', () => {
		const connectUrl =
			'wss://connect.steel.dev/session-123?sessionId=session-123&apiKey=secret-key&token=secret-token';

		expect(sanitizeConnectUrlForDisplay(connectUrl)).toBe(
			'wss://connect.steel.dev/session-123?sessionId=session-123&apiKey=REDACTED&token=REDACTED',
		);
	});

	test('falls back to regex sanitization for non-URL values', () => {
		expect(sanitizeConnectUrlForDisplay('connect?apiKey=secret-value')).toBe(
			'connect?apiKey=REDACTED',
		);
	});
});
