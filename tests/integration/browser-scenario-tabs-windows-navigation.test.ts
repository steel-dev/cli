import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

type CommandResult = {
	status: number;
	stdout: string;
	stderr: string;
	output: string;
};

const COMMAND_TIMEOUT_MS = 90_000;

function runBrowserCommand(
	arguments_: string[],
	environment: NodeJS.ProcessEnv,
	projectRoot: string,
): CommandResult {
	const result = spawnSync(
		process.execPath,
		['dist/steel.js', 'browser', ...arguments_],
		{
			cwd: projectRoot,
			env: environment,
			encoding: 'utf-8',
			timeout: COMMAND_TIMEOUT_MS,
			killSignal: 'SIGKILL',
		},
	);

	const stdout = result.stdout || '';
	const stderrParts = [result.stderr || ''];

	if (result.error) {
		stderrParts.push(`spawn error: ${result.error.message}`);
	}

	if (result.signal) {
		stderrParts.push(`terminated by signal: ${result.signal}`);
	}

	const stderr = stderrParts.filter(Boolean).join('\n');
	const output = [stdout, stderr].filter(Boolean).join('\n');

	return {
		status: result.status ?? 1,
		stdout,
		stderr,
		output,
	};
}

function assertSuccessfulStep(stepName: string, result: CommandResult): void {
	if (result.status === 0) {
		return;
	}

	throw new Error(
		[
			`${stepName} failed with exit code ${result.status}.`,
			result.stdout.trim() ? `stdout:\n${result.stdout.trim()}` : null,
			result.stderr.trim() ? `stderr:\n${result.stderr.trim()}` : null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

describe('browser tabs windows navigation scenario', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'handles tab/window controls with navigation state checks',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-tabs-windows-nav-'),
			);
			const sessionName = `steel-browser-tabs-windows-nav-${Date.now()}`;
			const environment = {
				...process.env,
				STEEL_API_KEY: apiKey!,
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
			};

			let sessionStarted = false;

			try {
				const startResult = runBrowserCommand(
					['start', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser start', startResult);
				sessionStarted = true;

				const openWindowsPageResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/windows',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open windows page',
					openWindowsPageResult,
				);

				const getWindowsUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get initial windows url',
					getWindowsUrlResult,
				);
				expect(getWindowsUrlResult.output).toContain('/windows');

				const openExampleTabResult = runBrowserCommand(
					['tab', 'new', 'https://example.com', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser tab new example', openExampleTabResult);

				const listTabsResult = runBrowserCommand(
					['tab', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser tab list', listTabsResult);
				expect(listTabsResult.output).toContain('example.com');
				expect(listTabsResult.output).toContain('/windows');

				const switchToExampleTabResult = runBrowserCommand(
					['tab', '1', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser switch to tab 1',
					switchToExampleTabResult,
				);

				const getExampleTitleResult = runBrowserCommand(
					['get', 'title', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get title in example tab',
					getExampleTitleResult,
				);
				expect(getExampleTitleResult.output).toContain('Example Domain');

				const switchToWindowsTabResult = runBrowserCommand(
					['tab', '0', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser switch back to tab 0',
					switchToWindowsTabResult,
				);

				const getWindowsUrlAgainResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get url after switching back to tab 0',
					getWindowsUrlAgainResult,
				);
				expect(getWindowsUrlAgainResult.output).toContain('/windows');

				const openNewWindowResult = runBrowserCommand(
					['window', 'new', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser window new', openNewWindowResult);

				const getNewWindowUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get url in new window context',
					getNewWindowUrlResult,
				);
				expect(getNewWindowUrlResult.output).toContain('about:blank');

				const openExampleInWindowResult = runBrowserCommand(
					['open', 'https://example.com', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open example in current window',
					openExampleInWindowResult,
				);

				const reloadExampleResult = runBrowserCommand(
					['reload', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser reload example page',
					reloadExampleResult,
				);

				const getReloadedTitleResult = runBrowserCommand(
					['get', 'title', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get title after reload',
					getReloadedTitleResult,
				);
				expect(getReloadedTitleResult.output).toContain('Example Domain');

				const openWindowsResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/windows',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser reopen windows page', openWindowsResult);

				const getFinalWindowsUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get final windows url',
					getFinalWindowsUrlResult,
				);
				expect(getFinalWindowsUrlResult.output).toContain('/windows');
			} finally {
				if (sessionStarted) {
					const stopResult = runBrowserCommand(
						['stop'],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser stop', stopResult);
				}

				fs.rmSync(configDirectory, {recursive: true, force: true});
			}
		},
		240_000,
	);
});
