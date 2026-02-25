import {
	bootstrapBrowserPassthroughArgv,
	parseBrowserPassthroughBootstrapFlags,
} from '../../source/utils/browser/lifecycle';
import {BrowserAdapterError} from '../../source/utils/browser/errors';

describe('browser lifecycle passthrough bootstrap parsing', () => {
	test('extracts bootstrap options and keeps runtime args', () => {
		const parsed = parseBrowserPassthroughBootstrapFlags([
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
			autoConnectUrl: null,
		});
		expect(parsed.passthroughArgv).toEqual([
			'open',
			'https://steel.dev',
			'--wait-for',
			'load',
		]);
	});

	test('throws for value flags without value', () => {
		expect(() =>
			parseBrowserPassthroughBootstrapFlags(['open', '--session']),
		).toThrow(BrowserAdapterError);
		expect(() =>
			parseBrowserPassthroughBootstrapFlags(['open', '--proxy']),
		).toThrow(BrowserAdapterError);
	});

	test('keeps explicit auto-connect untouched', async () => {
		const result = await bootstrapBrowserPassthroughArgv([
			'open',
			'https://steel.dev',
			'--auto-connect',
			'ws://localhost:9222',
		]);

		expect(result).toEqual([
			'open',
			'https://steel.dev',
			'--auto-connect',
			'ws://localhost:9222',
		]);
	});

	test('skips bootstrap for help output', async () => {
		const result = await bootstrapBrowserPassthroughArgv(['open', '--help']);

		expect(result).toEqual(['open', '--help']);
	});
});
