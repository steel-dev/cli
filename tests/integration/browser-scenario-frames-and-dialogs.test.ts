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

describe('browser frames and dialogs scenario', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'handles iframe commands and javascript alert/confirm/prompt dialogs',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-frames-dialogs-'),
			);
			const sessionName = `steel-browser-frames-dialogs-${Date.now()}`;
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

				const openIframePageResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/iframe',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open iframe page', openIframePageResult);

				const switchToIframeResult = runBrowserCommand(
					['frame', '#mce_0_ifr', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser switch to iframe', switchToIframeResult);

				const iframeSnapshotResult = runBrowserCommand(
					['snapshot', '-i', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser snapshot in iframe context',
					iframeSnapshotResult,
				);
				expect(iframeSnapshotResult.output).toContain('Powered by Tiny');
				expect(iframeSnapshotResult.output).toMatch(/ref=e\d+/);

				const switchToMainFrameResult = runBrowserCommand(
					['frame', 'main', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser switch to main frame',
					switchToMainFrameResult,
				);

				const openAlertsPageResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/javascript_alerts',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open javascript alerts page',
					openAlertsPageResult,
				);

				const clickAlertButtonResult = runBrowserCommand(
					['click', "button[onclick='jsAlert()']", '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser click js alert button',
					clickAlertButtonResult,
				);

				const acceptAlertResult = runBrowserCommand(
					['dialog', 'accept', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser dialog accept alert', acceptAlertResult);

				const getAlertResultText = runBrowserCommand(
					['get', 'text', '#result', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get alert result text',
					getAlertResultText,
				);
				expect(getAlertResultText.output).toContain(
					'You successfully clicked an alert',
				);

				const reopenAlertsForConfirmResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/javascript_alerts',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser reopen javascript alerts page for confirm',
					reopenAlertsForConfirmResult,
				);

				const clickConfirmButtonResult = runBrowserCommand(
					['click', "button[onclick='jsConfirm()']", '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser click js confirm button',
					clickConfirmButtonResult,
				);

				const acceptConfirmResult = runBrowserCommand(
					['dialog', 'accept', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser dialog accept confirm',
					acceptConfirmResult,
				);

				const getConfirmResultText = runBrowserCommand(
					['get', 'text', '#result', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get confirm result text',
					getConfirmResultText,
				);
				expect(getConfirmResultText.output).toContain('You clicked: Ok');
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
