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

function assertFailedStep(stepName: string, result: CommandResult): void {
	if (result.status !== 0) {
		return;
	}

	throw new Error(
		[
			`${stepName} unexpectedly succeeded with exit code 0.`,
			result.stdout.trim() ? `stdout:\n${result.stdout.trim()}` : null,
			result.stderr.trim() ? `stderr:\n${result.stderr.trim()}` : null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

describe('browser auth scenario login/logout', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'logs in and logs out successfully on practicetestautomation.com',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-auth-login-logout-'),
			);
			const sessionName = `steel-browser-auth-${Date.now()}`;
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

				const openLoginResult = runBrowserCommand(
					[
						'open',
						'https://practicetestautomation.com/practice-test-login/',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open login page', openLoginResult);

				const loginSnapshotResult = runBrowserCommand(
					['snapshot', '-i', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser snapshot login page',
					loginSnapshotResult,
				);
				expect(loginSnapshotResult.output).toContain('Username');
				expect(loginSnapshotResult.output).toContain('Password');
				expect(loginSnapshotResult.output).toMatch(/ref=e\d+/);

				const fillUsernameResult = runBrowserCommand(
					['fill', '#username', 'student', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill username', fillUsernameResult);

				const fillPasswordResult = runBrowserCommand(
					['fill', '#password', 'Password123', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill password', fillPasswordResult);

				const clickLoginResult = runBrowserCommand(
					['click', '#submit', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click login', clickLoginResult);

				const waitSuccessResult = runBrowserCommand(
					[
						'wait',
						'--text',
						'Logged In Successfully',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for secure area success text',
					waitSuccessResult,
				);

				const getSecureUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser get secure url', getSecureUrlResult);
				expect(getSecureUrlResult.output).toContain('/logged-in-successfully/');

				const getSecureFlashResult = runBrowserCommand(
					['get', 'text', '.post-title', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get secure flash text',
					getSecureFlashResult,
				);
				expect(getSecureFlashResult.output).toContain('Logged In Successfully');

				const clickLogoutResult = runBrowserCommand(
					['click', '.wp-block-button__link', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click logout', clickLogoutResult);

				const waitLoginUrlResult = runBrowserCommand(
					[
						'wait',
						'--url',
						'**/practice-test-login/',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for login url after logout',
					waitLoginUrlResult,
				);

				const getLoggedOutUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get logged-out url',
					getLoggedOutUrlResult,
				);
				expect(getLoggedOutUrlResult.output).toContain('/practice-test-login/');

				const getLoggedOutFlashResult = runBrowserCommand(
					['get', 'title', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get logged-out flash text',
					getLoggedOutFlashResult,
				);
				expect(getLoggedOutFlashResult.output).toContain('Test Login');
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
		120_000,
	);

	cloudTest(
		'fails login with bad credentials and exits non-zero on protected action',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-auth-bad-creds-'),
			);
			const sessionName = `steel-browser-auth-bad-creds-${Date.now()}`;
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

				const openLoginResult = runBrowserCommand(
					[
						'open',
						'https://practicetestautomation.com/practice-test-login/',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open login page', openLoginResult);

				const fillUsernameResult = runBrowserCommand(
					['fill', '#username', 'student', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill username', fillUsernameResult);

				const fillPasswordResult = runBrowserCommand(
					['fill', '#password', 'wrong-password', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill password', fillPasswordResult);

				const clickLoginResult = runBrowserCommand(
					['click', '#submit', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click login', clickLoginResult);

				const getInvalidFlashResult = runBrowserCommand(
					['get', 'text', '#error', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get invalid-credentials flash text',
					getInvalidFlashResult,
				);
				expect(getInvalidFlashResult.output).toContain(
					'Your password is invalid!',
				);

				const clickMissingLogoutResult = runBrowserCommand(
					['click', '.wp-block-button__link', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertFailedStep(
					'browser click logout after bad credentials',
					clickMissingLogoutResult,
				);
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
		120_000,
	);
});
