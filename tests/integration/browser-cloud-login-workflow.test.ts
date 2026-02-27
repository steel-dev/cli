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
		},
	);

	const stdout = result.stdout || '';
	const stderr = result.stderr || '';

	return {
		status: result.status ?? 1,
		stdout,
		stderr,
		output: `${stdout}${stderr}`,
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

describe('browser cloud login workflow e2e', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'executes a richer login workflow with click/fill/scroll/snapshot commands',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-cloud-login-e2e-'),
			);
			const sessionName = `steel-browser-login-e2e-${Date.now()}`;
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
				expect(loginSnapshotResult.output).toContain('Submit');
				expect(loginSnapshotResult.output).toMatch(/ref=e\d+/);

				const scrollDownResult = runBrowserCommand(
					['scroll', 'down', '300', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser scroll down', scrollDownResult);

				const scrollUpResult = runBrowserCommand(
					['scroll', 'up', '200', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser scroll up', scrollUpResult);

				const clickUsernameResult = runBrowserCommand(
					['click', '#username', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click username', clickUsernameResult);

				const fillUsernameResult = runBrowserCommand(
					['fill', '#username', 'student', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill username', fillUsernameResult);

				const clickPasswordResult = runBrowserCommand(
					['click', '#password', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click password', clickPasswordResult);

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
					'browser wait for login success',
					waitSuccessResult,
				);

				const getUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser get url', getUrlResult);
				expect(getUrlResult.output).toContain('/logged-in-successfully/');

				const getPageHeaderResult = runBrowserCommand(
					['get', 'text', '.post-title', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get logged-in page title text',
					getPageHeaderResult,
				);
				expect(getPageHeaderResult.output).toContain('Logged In Successfully');

				const getLinkCountResult = runBrowserCommand(
					['get', 'count', 'a', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser get link count', getLinkCountResult);
				const linkCount = Number.parseInt(getLinkCountResult.output.trim(), 10);
				expect(Number.isNaN(linkCount)).toBe(false);
				expect(linkCount).toBeGreaterThanOrEqual(1);

				const secureSnapshotResult = runBrowserCommand(
					['snapshot', '-i', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser snapshot logged-in page',
					secureSnapshotResult,
				);
				expect(secureSnapshotResult.output).toContain('Log out');
				expect(secureSnapshotResult.output).toMatch(/ref=e\d+/);

				const clickLogoutResult = runBrowserCommand(
					['click', '.wp-block-button__link', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click logout', clickLogoutResult);

				const waitLoginPageResult = runBrowserCommand(
					['wait', '--text', 'Test login', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for login page after logout',
					waitLoginPageResult,
				);

				const getLoggedOutUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get url after logout',
					getLoggedOutUrlResult,
				);
				expect(getLoggedOutUrlResult.output).toContain('/practice-test-login/');
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
		180_000,
	);
});
