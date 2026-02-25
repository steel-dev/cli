import {
	filterSteelGlobalFlags,
	getBrowserPassthroughArgv,
	isBrowserCommand,
	resolveBrowserDispatchTarget,
} from '../../source/utils/browser/routing';

describe('browser routing', () => {
	test('detects browser command path', () => {
		expect(isBrowserCommand(['browser', 'start'])).toBe(true);
		expect(isBrowserCommand(['run'])).toBe(false);
	});

	test('routes native browser lifecycle commands to pastel', () => {
		expect(resolveBrowserDispatchTarget(['browser', 'start'])).toBe('native');
		expect(resolveBrowserDispatchTarget(['browser', 'stop'])).toBe('native');
		expect(resolveBrowserDispatchTarget(['browser', 'sessions'])).toBe(
			'native',
		);
		expect(resolveBrowserDispatchTarget(['browser', 'live'])).toBe('native');
	});

	test('routes inherited browser commands to passthrough', () => {
		expect(resolveBrowserDispatchTarget(['browser', 'open'])).toBe(
			'passthrough',
		);
		expect(resolveBrowserDispatchTarget(['browser', 'snapshot', '-i'])).toBe(
			'passthrough',
		);
	});

	test('filters steel global flags before routing', () => {
		expect(
			resolveBrowserDispatchTarget([
				'--no-update-check',
				'browser',
				'open',
				'https://example.com',
			]),
		).toBe('passthrough');

		expect(
			filterSteelGlobalFlags([
				'--no-update-check',
				'browser',
				'open',
				'https://example.com',
			]),
		).toEqual(['browser', 'open', 'https://example.com']);
	});

	test('extracts passthrough argv while preserving browser command args', () => {
		expect(
			getBrowserPassthroughArgv([
				'--no-update-check',
				'browser',
				'open',
				'https://steel.dev',
				'--wait-for',
				'load',
			]),
		).toEqual(['open', 'https://steel.dev', '--wait-for', 'load']);
	});
});
