import {describe, test, expect} from 'vitest';
import type {BrowserSessionSummary} from '../../source/utils/browser/lifecycle';
import {formatBrowserSessionsForOutput} from '../../source/utils/browser/display';

const baseSession: BrowserSessionSummary = {
	id: 'session-123',
	mode: 'cloud',
	name: 'daily',
	live: true,
	status: 'live',
	connectUrl:
		'wss://connect.steel.dev?sessionId=session-123&apiKey=secret-key&token=secret-token',
	viewerUrl: 'https://app.steel.dev/sessions/session-123',
	raw: {
		id: 'session-123',
		status: 'live',
		metadata: {
			labels: ['daily'],
		},
	},
};

describe('browser sessions output formatting', () => {
	test('omits raw session payload by default', () => {
		const output = formatBrowserSessionsForOutput([baseSession]);

		expect(output).toEqual({
			sessions: [
				{
					id: 'session-123',
					mode: 'cloud',
					name: 'daily',
					live: true,
					status: 'live',
					connectUrl:
						'wss://connect.steel.dev/?sessionId=session-123&apiKey=REDACTED&token=REDACTED',
					viewerUrl: 'https://app.steel.dev/sessions/session-123',
				},
			],
		});
		expect(output.sessions[0]).not.toHaveProperty('raw');
	});

	test('includes raw session payload when raw option is enabled', () => {
		const output = formatBrowserSessionsForOutput([baseSession], {raw: true});

		expect(output).toEqual({
			sessions: [
				{
					id: 'session-123',
					mode: 'cloud',
					name: 'daily',
					live: true,
					status: 'live',
					connectUrl:
						'wss://connect.steel.dev/?sessionId=session-123&apiKey=REDACTED&token=REDACTED',
					viewerUrl: 'https://app.steel.dev/sessions/session-123',
					raw: {
						id: 'session-123',
						status: 'live',
						metadata: {
							labels: ['daily'],
						},
					},
				},
			],
		});
	});
});
