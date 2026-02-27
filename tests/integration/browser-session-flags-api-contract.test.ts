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

type UnknownRecord = Record<string, unknown>;
const ANSI_ESCAPE_PATTERN = new RegExp(
	`${String.fromCharCode(27)}\\[[0-9;]*m`,
	'g',
);

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

function stripAnsi(value: string): string {
	return value.replace(ANSI_ESCAPE_PATTERN, '');
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
	const normalizedOutput = stripAnsi(commandOutput);
	const sessionIdMatch = normalizedOutput.match(/(?:^|\n)id:\s*([^\s]+)/);
	if (!sessionIdMatch?.[1]) {
		throw new Error(
			[
				'Could not parse session id from browser start output.',
				normalizedOutput.trim() ? normalizedOutput.trim() : '(empty output)',
			].join('\n'),
		);
	}

	return sessionIdMatch[1];
}

function assertConnectUrlIsRedacted(commandOutput: string): void {
	const normalizedOutput = stripAnsi(commandOutput);
	const connectUrlMatch = normalizedOutput.match(
		/(?:^|\n)connect_url:\s*([^\s]+)/,
	);
	if (!connectUrlMatch?.[1]) {
		return;
	}

	expect(connectUrlMatch[1]).not.toMatch(
		/[?&](?:apiKey|api_key|token|access_token)=(?!REDACTED\b)[^&]+/i,
	);
}

function asRecord(value: unknown): UnknownRecord | null {
	if (!value || typeof value !== 'object' || Array.isArray(value)) {
		return null;
	}

	return value as UnknownRecord;
}

async function getSessionById(
	sessionId: string,
	apiKey: string,
): Promise<UnknownRecord> {
	for (let attempt = 0; attempt < 5; attempt++) {
		const response = await fetch(
			`https://api.steel.dev/v1/sessions/${sessionId}`,
			{
				headers: {
					'Steel-Api-Key': apiKey,
					'Content-Type': 'application/json',
				},
			},
		);
		const bodyText = await response.text();

		if (response.status === 404 && attempt < 4) {
			await new Promise(resolve => {
				setTimeout(resolve, 400);
			});
			continue;
		}

		if (!response.ok) {
			throw new Error(
				[
					`Failed to fetch session ${sessionId} (${response.status}).`,
					bodyText.trim() ? `body:\n${bodyText.trim()}` : null,
				]
					.filter(Boolean)
					.join('\n'),
			);
		}

		if (!bodyText.trim()) {
			throw new Error(`Session details response for ${sessionId} was empty.`);
		}

		const parsedBody = JSON.parse(bodyText) as unknown;
		const topLevel = asRecord(parsedBody);
		if (!topLevel) {
			throw new Error(
				`Session details for ${sessionId} were not a JSON object.`,
			);
		}

		const nestedSession = asRecord(topLevel['session']);
		return nestedSession || topLevel;
	}

	throw new Error(
		`Failed to fetch session ${sessionId} because it remained unavailable.`,
	);
}

async function releaseSessionById(
	sessionId: string,
	apiKey: string,
): Promise<void> {
	const response = await fetch(
		`https://api.steel.dev/v1/sessions/${sessionId}/release`,
		{
			method: 'POST',
			headers: {
				'Steel-Api-Key': apiKey,
				'Content-Type': 'application/json',
			},
		},
	);

	if (response.ok) {
		return;
	}

	const bodyText = await response.text();
	throw new Error(
		[
			`Failed to release session ${sessionId} (${response.status}).`,
			bodyText.trim() ? `body:\n${bodyText.trim()}` : null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

describe('browser session flag mapping contract', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'maps --stealth and --proxy flags to expected Steel Sessions API fields',
		async () => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-session-flags-contract-'),
			);
			const environment = {
				...process.env,
				STEEL_API_KEY: apiKey!,
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
			};

			const createdSessionIds = new Set<string>();

			try {
				const baselineName = `steel-browser-flags-baseline-${Date.now()}`;
				const baselineStartResult = runBrowserCommand(
					['start', '--session', baselineName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('baseline browser start', baselineStartResult);
				assertConnectUrlIsRedacted(baselineStartResult.output);

				const baselineSessionId = extractSessionId(baselineStartResult.output);
				createdSessionIds.add(baselineSessionId);

				const baselineSession = await getSessionById(
					baselineSessionId,
					apiKey!,
				);
				const baselineStealthConfig = asRecord(
					baselineSession['stealthConfig'],
				);

				expect(baselineSession['proxySource']).not.toBe('external');
				expect(baselineStealthConfig?.['humanizeInteractions']).not.toBe(true);
				expect(baselineStealthConfig?.['autoCaptchaSolving']).not.toBe(true);
				expect(baselineSession['solveCaptcha']).not.toBe(true);

				const stopBaselineResult = runBrowserCommand(
					['stop'],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('baseline browser stop', stopBaselineResult);

				const flaggedName = `steel-browser-flags-stealth-proxy-${Date.now()}`;
				const flaggedStartResult = runBrowserCommand(
					[
						'start',
						'--session',
						flaggedName,
						'--stealth',
						'--proxy',
						'http://127.0.0.1:8080',
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('flagged browser start', flaggedStartResult);
				assertConnectUrlIsRedacted(flaggedStartResult.output);

				const flaggedSessionId = extractSessionId(flaggedStartResult.output);
				createdSessionIds.add(flaggedSessionId);

				const flaggedSession = await getSessionById(flaggedSessionId, apiKey!);
				const flaggedStealthConfig = asRecord(flaggedSession['stealthConfig']);

				expect(flaggedSession['proxySource']).toBe('external');
				expect(flaggedStealthConfig?.['humanizeInteractions']).toBe(true);
				expect(flaggedStealthConfig?.['autoCaptchaSolving']).toBe(true);
				expect(flaggedSession['solveCaptcha']).toBe(true);
			} finally {
				const stopResult = runBrowserCommand(
					['stop'],
					environment,
					projectRoot,
				);
				if (stopResult.status !== 0) {
					console.warn(
						`Cleanup warning: browser stop failed with status ${stopResult.status}.`,
					);
				}

				for (const sessionId of createdSessionIds) {
					try {
						await releaseSessionById(sessionId, apiKey!);
					} catch {
						// Best effort cleanup.
					}
				}

				fs.rmSync(configDirectory, {recursive: true, force: true});
			}
		},
		120_000,
	);
});
