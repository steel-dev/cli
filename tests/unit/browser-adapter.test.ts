import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	requiresBrowserAuth,
	resolveBrowserRuntime,
} from '../../source/utils/browser/adapter';
import {resolveBrowserAuth} from '../../source/utils/browser/auth';

describe('browser adapter auth contract', () => {
	test('prefers STEEL_API_KEY from environment over config value', () => {
		const result = resolveBrowserAuth(
			{
				STEEL_API_KEY: 'env-api-key',
			},
			'config-api-key',
		);

		expect(result).toEqual({
			apiKey: 'env-api-key',
			source: 'env',
		});
	});

	test('falls back to config api key when environment key is missing', () => {
		const result = resolveBrowserAuth({}, 'config-api-key');

		expect(result).toEqual({
			apiKey: 'config-api-key',
			source: 'config',
		});
	});

	test('returns no auth when both environment and config are empty', () => {
		const result = resolveBrowserAuth({}, null);

		expect(result).toEqual({
			apiKey: null,
			source: 'none',
		});
	});
});

describe('browser adapter routing contract', () => {
	test('does not require auth for local or direct auto-connect usage', () => {
		expect(
			requiresBrowserAuth(['open', 'https://example.com', '--local']),
		).toBe(false);
		expect(
			requiresBrowserAuth([
				'open',
				'https://example.com',
				'--auto-connect',
				'ws://localhost:9222',
			]),
		).toBe(false);
	});

	test('does not require auth for help output', () => {
		expect(requiresBrowserAuth(['open', '--help'])).toBe(false);
		expect(requiresBrowserAuth(['open', '-h'])).toBe(false);
	});

	test('requires auth for default cloud path', () => {
		expect(requiresBrowserAuth(['open', 'https://steel.dev'])).toBe(true);
	});

	test('uses path runtime fallback when no runtime override is configured', () => {
		const temporaryRoot = fs.mkdtempSync(
			path.join(os.tmpdir(), 'steel-browser-runtime-fallback-'),
		);
		const previousCwd = process.cwd();

		try {
			process.chdir(temporaryRoot);
			const runtime = resolveBrowserRuntime({});

			expect(runtime).toEqual({
				command: 'agent-browser',
				args: [],
				source: 'path',
			});
		} finally {
			process.chdir(previousCwd);
			fs.rmSync(temporaryRoot, {recursive: true, force: true});
		}
	});

	test('uses runtime override from environment', () => {
		const runtime = resolveBrowserRuntime({
			STEEL_BROWSER_RUNTIME_BIN: '/tmp/browser-runtime.js',
		});

		expect(runtime).toEqual({
			command: process.execPath,
			args: ['/tmp/browser-runtime.js'],
			source: 'env',
		});
	});
});
