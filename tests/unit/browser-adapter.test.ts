import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	requiresBrowserAuth,
	resolveBrowserRuntime,
	runBrowserPassthrough,
} from '../../source/utils/browser/adapter';
import {resolveBrowserAuth} from '../../source/utils/browser/auth';
import {getBrowserRuntimeTarget} from '../../source/utils/browser/runtime';

function createTempDirectory(prefix: string): string {
	return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function writeProbeRuntimeScript(rootDirectory: string): {
	runtimePath: string;
	capturePath: string;
} {
	const runtimePath = path.join(rootDirectory, 'runtime-probe.mjs');
	const capturePath = path.join(rootDirectory, 'runtime-probe-output.json');

	fs.writeFileSync(
		runtimePath,
		[
			"import fs from 'node:fs';",
			'const payload = {',
			'  argv: process.argv.slice(2),',
			'  apiKey: process.env.STEEL_API_KEY ?? null,',
			'  agentBrowserHome: process.env.AGENT_BROWSER_HOME ?? null,',
			'};',
			'if (process.env.STEEL_RUNTIME_CAPTURE_PATH) {',
			'  fs.writeFileSync(',
			'    process.env.STEEL_RUNTIME_CAPTURE_PATH,',
			'    JSON.stringify(payload),',
			"    'utf-8',",
			'  );',
			'}',
			'const parsedExitCode = Number.parseInt(',
			"  process.env.STEEL_RUNTIME_EXIT_CODE ?? '0',",
			'  10,',
			');',
			'process.exit(Number.isNaN(parsedExitCode) ? 1 : parsedExitCode);',
			'',
		].join('\n'),
		'utf-8',
	);

	return {
		runtimePath,
		capturePath,
	};
}

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
	test('does not require auth for local or direct attach usage', () => {
		expect(
			requiresBrowserAuth(['open', 'https://example.com', '--local']),
		).toBe(false);
		expect(
			requiresBrowserAuth(['open', 'https://example.com', '--auto-connect']),
		).toBe(false);
		expect(
			requiresBrowserAuth([
				'open',
				'https://example.com',
				'--cdp',
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

	test('uses a vendored runtime binary from manifest when available', () => {
		const runtimeTarget = getBrowserRuntimeTarget();
		if (!runtimeTarget) {
			return;
		}

		const temporaryRoot = createTempDirectory(
			'steel-browser-runtime-vendored-',
		);
		const previousCwd = process.cwd();

		try {
			const runtimePath = path.join(
				temporaryRoot,
				'vendor/agent-browser/runtimes',
				runtimeTarget,
				process.platform === 'win32' ? 'agent-browser.exe' : 'agent-browser',
			);
			const manifestPath = path.join(
				temporaryRoot,
				'vendor/agent-browser/runtime-manifest.json',
			);

			fs.mkdirSync(path.dirname(runtimePath), {recursive: true});
			fs.writeFileSync(runtimePath, 'runtime', 'utf-8');
			fs.writeFileSync(
				manifestPath,
				JSON.stringify(
					{
						schemaVersion: 1,
						runtimeVersion: 'test',
						platforms: {
							[runtimeTarget]: {
								entrypoint: path
									.relative(path.dirname(manifestPath), runtimePath)
									.split(path.sep)
									.join('/'),
							},
						},
					},
					null,
					2,
				),
				'utf-8',
			);

			process.chdir(temporaryRoot);
			const runtime = resolveBrowserRuntime({});

			expect(runtime.source).toBe('vendored');
			expect(runtime.args).toEqual([]);
			expect(fs.realpathSync(runtime.command)).toBe(
				fs.realpathSync(runtimePath),
			);
		} finally {
			process.chdir(previousCwd);
			fs.rmSync(temporaryRoot, {recursive: true, force: true});
		}
	});
});

describe('browser passthrough execution contract', () => {
	test('preserves passthrough argv and exit code while injecting config auth', async () => {
		const temporaryRoot = createTempDirectory(
			'steel-browser-passthrough-runtime-',
		);
		const configDirectory = path.join(temporaryRoot, 'config');
		const configPath = path.join(configDirectory, 'config.json');
		const {runtimePath, capturePath} = writeProbeRuntimeScript(temporaryRoot);

		try {
			fs.mkdirSync(configDirectory, {recursive: true});
			fs.writeFileSync(
				configPath,
				JSON.stringify(
					{
						apiKey: 'config-api-key',
					},
					null,
					2,
				),
				'utf-8',
			);

			const exitCode = await runBrowserPassthrough(['open', '--help'], {
				STEEL_BROWSER_RUNTIME_BIN: runtimePath,
				STEEL_RUNTIME_CAPTURE_PATH: capturePath,
				STEEL_RUNTIME_EXIT_CODE: '17',
				STEEL_CONFIG_DIR: configDirectory,
			});
			const runtimePayload = JSON.parse(
				fs.readFileSync(capturePath, 'utf-8'),
			) as {
				argv: string[];
				apiKey: string | null;
				agentBrowserHome: string | null;
			};

			expect(exitCode).toBe(17);
			expect(runtimePayload).toEqual({
				argv: ['open', '--help'],
				apiKey: 'config-api-key',
				agentBrowserHome: null,
			});
		} finally {
			fs.rmSync(temporaryRoot, {recursive: true, force: true});
		}
	});

	test('keeps environment api key precedence when spawning passthrough runtime', async () => {
		const temporaryRoot = createTempDirectory(
			'steel-browser-passthrough-auth-',
		);
		const {runtimePath, capturePath} = writeProbeRuntimeScript(temporaryRoot);

		try {
			const exitCode = await runBrowserPassthrough(['open', '--help'], {
				STEEL_BROWSER_RUNTIME_BIN: runtimePath,
				STEEL_RUNTIME_CAPTURE_PATH: capturePath,
				STEEL_API_KEY: 'env-api-key',
			});
			const runtimePayload = JSON.parse(
				fs.readFileSync(capturePath, 'utf-8'),
			) as {
				argv: string[];
				apiKey: string | null;
				agentBrowserHome: string | null;
			};

			expect(exitCode).toBe(0);
			expect(runtimePayload.apiKey).toBe('env-api-key');
			expect(runtimePayload.agentBrowserHome).toBeNull();
		} finally {
			fs.rmSync(temporaryRoot, {recursive: true, force: true});
		}
	});

	test('sets AGENT_BROWSER_HOME for vendored runtime execution', async () => {
		const runtimeTarget = getBrowserRuntimeTarget();
		if (!runtimeTarget) {
			return;
		}

		const temporaryRoot = createTempDirectory(
			'steel-browser-passthrough-vendored-runtime-',
		);
		const configDirectory = path.join(temporaryRoot, 'config');
		const manifestPath = path.join(
			temporaryRoot,
			'vendor/agent-browser/runtime-manifest.json',
		);
		const runtimePath = path.join(
			temporaryRoot,
			'vendor/agent-browser/runtimes',
			runtimeTarget,
			'cli.mjs',
		);
		const daemonPath = path.join(
			temporaryRoot,
			'vendor/agent-browser/dist/daemon.js',
		);
		const capturePath = path.join(
			temporaryRoot,
			'vendored-runtime-output.json',
		);
		const previousCwd = process.cwd();

		try {
			fs.mkdirSync(path.dirname(runtimePath), {recursive: true});
			fs.mkdirSync(path.dirname(daemonPath), {recursive: true});
			fs.mkdirSync(configDirectory, {recursive: true});
			fs.writeFileSync(
				runtimePath,
				[
					"import fs from 'node:fs';",
					'const payload = {',
					'  argv: process.argv.slice(2),',
					'  agentBrowserHome: process.env.AGENT_BROWSER_HOME ?? null,',
					'};',
					'if (process.env.STEEL_RUNTIME_CAPTURE_PATH) {',
					'  fs.writeFileSync(',
					'    process.env.STEEL_RUNTIME_CAPTURE_PATH,',
					'    JSON.stringify(payload),',
					"    'utf-8',",
					'  );',
					'}',
					'process.exit(0);',
					'',
				].join('\n'),
				'utf-8',
			);
			fs.writeFileSync(daemonPath, 'export {};\n', 'utf-8');
			fs.writeFileSync(
				manifestPath,
				JSON.stringify(
					{
						schemaVersion: 1,
						runtimeVersion: 'test',
						platforms: {
							[runtimeTarget]: {
								entrypoint: path
									.relative(path.dirname(manifestPath), runtimePath)
									.split(path.sep)
									.join('/'),
							},
						},
						shared: ['dist'],
					},
					null,
					2,
				),
				'utf-8',
			);

			process.chdir(temporaryRoot);
			const exitCode = await runBrowserPassthrough(['open', '--help'], {
				STEEL_RUNTIME_CAPTURE_PATH: capturePath,
				STEEL_CONFIG_DIR: configDirectory,
			});
			const runtimePayload = JSON.parse(
				fs.readFileSync(capturePath, 'utf-8'),
			) as {
				argv: string[];
				agentBrowserHome: string | null;
			};

			expect(exitCode).toBe(0);
			expect(runtimePayload.argv).toEqual(['open', '--help']);
			expect(fs.realpathSync(runtimePayload.agentBrowserHome || '')).toBe(
				fs.realpathSync(path.join(temporaryRoot, 'vendor/agent-browser')),
			);
		} finally {
			process.chdir(previousCwd);
			fs.rmSync(temporaryRoot, {recursive: true, force: true});
		}
	});

	test('throws runtime-not-found when configured runtime path is missing', async () => {
		await expect(
			runBrowserPassthrough(['open', '--help'], {
				STEEL_BROWSER_RUNTIME_BIN: '/tmp/does-not-exist/runtime-bin',
			}),
		).rejects.toMatchObject({
			name: 'BrowserAdapterError',
			code: 'RUNTIME_NOT_FOUND',
		});
	});
});
