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

function extractSessionId(commandOutput: string): string {
	const sessionIdMatch = commandOutput.match(/(?:^|\n)id:\s*([^\s]+)/);
	if (!sessionIdMatch?.[1]) {
		throw new Error(
			[
				'Could not parse session id from browser start output.',
				commandOutput.trim() ? commandOutput.trim() : '(empty output)',
			].join('\n'),
		);
	}

	return sessionIdMatch[1];
}

describe('browser scenario session reattach cross process', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'reattaches a named session across process boundaries and preserves page state',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-scenario-session-reattach-'),
			);
			const sessionName = `steel-browser-scenario-reattach-${Date.now()}`;
			const environmentProcessA = {
				...process.env,
				STEEL_API_KEY: apiKey!,
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
			};
			const environmentProcessB = {
				...environmentProcessA,
			};

			let sessionStarted = false;

			try {
				// Process A: create session and establish state.
				const startProcessAResult = runBrowserCommand(
					['start', '--session', sessionName],
					environmentProcessA,
					projectRoot,
				);
				assertSuccessfulStep('process A browser start', startProcessAResult);
				sessionStarted = true;

				const processASessionId = extractSessionId(startProcessAResult.output);

				const openLoginProcessAResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/login',
						'--session',
						sessionName,
					],
					environmentProcessA,
					projectRoot,
				);
				assertSuccessfulStep(
					'process A browser open login page',
					openLoginProcessAResult,
				);

				const waitLoginProcessAResult = runBrowserCommand(
					['wait', '--url', '**/login', '--session', sessionName],
					environmentProcessA,
					projectRoot,
				);
				assertSuccessfulStep(
					'process A browser wait for login url',
					waitLoginProcessAResult,
				);

				const fillUsernameProcessAResult = runBrowserCommand(
					['fill', '#username', 'tomsmith', '--session', sessionName],
					environmentProcessA,
					projectRoot,
				);
				assertSuccessfulStep(
					'process A browser fill username',
					fillUsernameProcessAResult,
				);

				const getUsernameValueProcessAResult = runBrowserCommand(
					['get', 'value', '#username', '--session', sessionName],
					environmentProcessA,
					projectRoot,
				);
				assertSuccessfulStep(
					'process A browser get username value',
					getUsernameValueProcessAResult,
				);
				expect(getUsernameValueProcessAResult.output).toContain('tomsmith');

				// Process B: reattach using the same named session and continue.
				const startProcessBResult = runBrowserCommand(
					['start', '--session', sessionName],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep('process B browser start', startProcessBResult);

				const processBSessionId = extractSessionId(startProcessBResult.output);
				expect(processBSessionId).toBe(processASessionId);

				const getUrlProcessBResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser get current url',
					getUrlProcessBResult,
				);
				expect(getUrlProcessBResult.output).toContain('/login');

				const getUsernameValueProcessBResult = runBrowserCommand(
					['get', 'value', '#username', '--session', sessionName],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser get preserved username value',
					getUsernameValueProcessBResult,
				);
				expect(getUsernameValueProcessBResult.output).toContain('tomsmith');

				const fillPasswordProcessBResult = runBrowserCommand(
					[
						'fill',
						'#password',
						'SuperSecretPassword!',
						'--session',
						sessionName,
					],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser fill password',
					fillPasswordProcessBResult,
				);

				const clickLoginProcessBResult = runBrowserCommand(
					['click', "button[type='submit']", '--session', sessionName],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser click login',
					clickLoginProcessBResult,
				);

				const waitSecureAreaProcessBResult = runBrowserCommand(
					[
						'wait',
						'--text',
						'You logged into a secure area!',
						'--session',
						sessionName,
					],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser wait for secure area message',
					waitSecureAreaProcessBResult,
				);

				const getSecureUrlProcessBResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environmentProcessB,
					projectRoot,
				);
				assertSuccessfulStep(
					'process B browser get secure url',
					getSecureUrlProcessBResult,
				);
				expect(getSecureUrlProcessBResult.output).toContain('/secure');
			} finally {
				if (sessionStarted) {
					const stopResult = runBrowserCommand(
						['stop'],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep('browser stop', stopResult);
				}

				fs.rmSync(configDirectory, {recursive: true, force: true});
			}
		},
		180_000,
	);
});
