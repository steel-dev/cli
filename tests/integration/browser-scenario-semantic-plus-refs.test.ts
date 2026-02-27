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

const TRANSIENT_BROWSER_ERRORS = [
	'daemon may be busy or unresponsive',
	'Target page, context or browser has been closed',
	'net::ERR_ABORTED',
	'Failed to connect: No such file or directory (os error 2)',
];

function hasTransientBrowserError(result: CommandResult): boolean {
	return (
		result.status !== 0 &&
		TRANSIENT_BROWSER_ERRORS.some(pattern => result.output.includes(pattern))
	);
}

function runBrowserCommand(
	arguments_: string[],
	environment: NodeJS.ProcessEnv,
	projectRoot: string,
): CommandResult {
	const attempts = 3;
	let latestResult: CommandResult = {
		status: 1,
		stdout: '',
		stderr: '',
		output: '',
	};

	for (let attempt = 1; attempt <= attempts; attempt += 1) {
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

		latestResult = {
			status: result.status ?? 1,
			stdout,
			stderr,
			output,
		};

		if (!hasTransientBrowserError(latestResult)) {
			return latestResult;
		}
	}

	return latestResult;
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

function extractRefForText(snapshotOutput: string, targetText: string): string {
	const targetLower = targetText.toLowerCase();

	for (const line of snapshotOutput.split('\n')) {
		if (!line.toLowerCase().includes(targetLower)) {
			continue;
		}

		const refMatch = line.match(/ref=(e\d+)/i);
		if (refMatch?.[1]) {
			return `@${refMatch[1]}`;
		}
	}

	throw new Error(
		`Could not find a ref for "${targetText}" in snapshot output.\n${snapshotOutput}`,
	);
}

describe('browser semantic locators + refs scenario', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'mixes snapshot refs with semantic role/text/label/placeholder locators',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-semantic-plus-refs-'),
			);
			const sessionName = `steel-browser-semantic-plus-refs-${Date.now()}`;
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

				const openPracticeLoginResult = runBrowserCommand(
					[
						'open',
						'https://practicetestautomation.com/practice-test-login/',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open practice login page',
					openPracticeLoginResult,
				);

				const snapshotResult = runBrowserCommand(
					['snapshot', '-i', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser snapshot -i', snapshotResult);
				expect(snapshotResult.output).toContain('Username');
				expect(snapshotResult.output).toMatch(/ref=e\d+/);

				const usernameRef = extractRefForText(
					snapshotResult.output,
					'Username',
				);

				const fillUsernameByRefResult = runBrowserCommand(
					['fill', usernameRef, 'student', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser fill username via ref',
					fillUsernameByRefResult,
				);

				const fillPasswordByLabelResult = runBrowserCommand(
					[
						'find',
						'label',
						'Password',
						'fill',
						'Password123',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser find label password fill',
					fillPasswordByLabelResult,
				);

				const clickSubmitByRoleResult = runBrowserCommand(
					[
						'find',
						'role',
						'button',
						'click',
						'--name',
						'Submit',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser find role button click submit',
					clickSubmitByRoleResult,
				);

				const waitLoggedInResult = runBrowserCommand(
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
				assertSuccessfulStep('browser wait logged-in text', waitLoggedInResult);

				const clickLogoutByTextResult = runBrowserCommand(
					[
						'find',
						'text',
						'Log out',
						'click',
						'--exact',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser find text logout click',
					clickLogoutByTextResult,
				);

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
					'browser wait login url after logout',
					waitLoginUrlResult,
				);

				const openSauceDemoResult = runBrowserCommand(
					['open', 'https://www.saucedemo.com/', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open saucedemo', openSauceDemoResult);

				const fillByPlaceholderResult = runBrowserCommand(
					[
						'find',
						'placeholder',
						'Username',
						'fill',
						'standard_user',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser find placeholder username fill',
					fillByPlaceholderResult,
				);

				const getSauceUsernameValueResult = runBrowserCommand(
					['get', 'value', '#user-name', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get saucedemo username value',
					getSauceUsernameValueResult,
				);
				expect(getSauceUsernameValueResult.output).toContain('standard_user');
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
