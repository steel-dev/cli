import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {test} from 'vitest';

export type CommandResult = {
	status: number;
	stdout: string;
	stderr: string;
	output: string;
};

const DEFAULT_COMMAND_TIMEOUT_MS = 90_000;
const SLOWEST_STEP_COUNT = 5;
const DEFAULT_TRANSIENT_BROWSER_ERRORS = [
	'daemon may be busy or unresponsive',
	'Target page, context or browser has been closed',
	'net::ERR_ABORTED',
	'Failed to connect: No such file or directory (os error 2)',
];
const ANSI_ESCAPE_PATTERN = new RegExp(
	`${String.fromCharCode(27)}\\[[0-9;]*m`,
	'g',
);

type RunBrowserCommandOptions = {
	environment: NodeJS.ProcessEnv;
	projectRoot: string;
	timeoutMs?: number;
	attempts?: number;
	transientErrorPatterns?: string[];
};

export type CloudHarness = {
	apiKey: string | null;
	cloudTest: typeof test;
	projectRoot: string;
	distEntrypoint: string;
	ensureBuilt: () => void;
	createEnvironment: (
		configDirectory: string,
		overrides?: NodeJS.ProcessEnv,
	) => NodeJS.ProcessEnv;
};

type CloudSessionOptions = {
	configDirectoryPrefix: string;
	sessionNamePrefix: string;
	environmentOverrides?: NodeJS.ProcessEnv;
	commandTimeoutMs?: number;
	commandAttempts?: number;
	transientErrorPatterns?: string[];
	cleanupStopBehavior?: 'assert' | 'warn' | 'ignore';
};

export type CloudSessionContext = {
	projectRoot: string;
	configDirectory: string;
	sessionName: string;
	environment: NodeJS.ProcessEnv;
	runCommand: (arguments_: string[]) => CommandResult;
	runStep: (stepName: string, arguments_: string[]) => CommandResult;
	runFailStep: (stepName: string, arguments_: string[]) => CommandResult;
	startSession: (stepName?: string) => CommandResult;
	stopSession: (stepName?: string) => CommandResult;
};

export type LegacyRunBrowserCommand = (
	arguments_: string[],
	environment: NodeJS.ProcessEnv,
	projectRoot: string,
) => CommandResult;

type StepTiming = {
	stepName: string;
	durationMs: number;
	status: number;
};

function isStepProfilingEnabled(): boolean {
	const value =
		process.env.STEEL_INTEGRATION_STEP_PROFILE?.trim().toLowerCase();
	return value === '1' || value === 'true' || value === 'yes';
}

export function runBrowserCommand(
	arguments_: string[],
	options: RunBrowserCommandOptions,
): CommandResult {
	const attempts = options.attempts || 1;
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
				cwd: options.projectRoot,
				env: options.environment,
				encoding: 'utf-8',
				timeout: options.timeoutMs || DEFAULT_COMMAND_TIMEOUT_MS,
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

		if (latestResult.status === 0) {
			return latestResult;
		}

		const transientErrorPatterns =
			options.transientErrorPatterns || DEFAULT_TRANSIENT_BROWSER_ERRORS;
		const hasTransientError = transientErrorPatterns.some(pattern =>
			latestResult.output.includes(pattern),
		);
		if (!hasTransientError) {
			return latestResult;
		}
	}

	return latestResult;
}

function formatStepFailure(header: string, result: CommandResult): string {
	return [
		header,
		result.stdout.trim() ? `stdout:\n${result.stdout.trim()}` : null,
		result.stderr.trim() ? `stderr:\n${result.stderr.trim()}` : null,
	]
		.filter(Boolean)
		.join('\n');
}

export function assertSuccessfulStep(
	stepName: string,
	result: CommandResult,
): void {
	if (result.status === 0) {
		return;
	}

	throw new Error(
		formatStepFailure(
			`${stepName} failed with exit code ${result.status}.`,
			result,
		),
	);
}

export function assertFailedStep(
	stepName: string,
	result: CommandResult,
): void {
	if (result.status !== 0) {
		return;
	}

	throw new Error(
		formatStepFailure(
			`${stepName} unexpectedly succeeded with exit code 0.`,
			result,
		),
	);
}

export function stripAnsi(value: string): string {
	return value.replace(ANSI_ESCAPE_PATTERN, '');
}

export function extractSessionId(commandOutput: string): string {
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

export function extractRefForText(
	snapshotOutput: string,
	targetText: string,
): string {
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

export function createLegacyRunBrowserCommand(
	runCommand: (arguments_: string[]) => CommandResult,
): LegacyRunBrowserCommand {
	return (arguments_, environment, projectRoot) => {
		void environment;
		void projectRoot;
		return runCommand(arguments_);
	};
}

export function createCloudHarness(importMetaUrl: string): CloudHarness {
	const testDirectory = path.dirname(fileURLToPath(importMetaUrl));
	const derivedProjectRoot = path.resolve(testDirectory, '../..');
	const cwdProjectRoot = process.cwd();
	const derivedDistEntrypoint = path.join(derivedProjectRoot, 'dist/steel.js');
	const cwdDistEntrypoint = path.join(cwdProjectRoot, 'dist/steel.js');
	const projectRoot = fs.existsSync(derivedDistEntrypoint)
		? derivedProjectRoot
		: cwdProjectRoot;
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim() || null;

	return {
		apiKey,
		cloudTest: apiKey ? test : test.skip,
		projectRoot,
		distEntrypoint,
		ensureBuilt() {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error(
					[
						'dist/steel.js is missing. Run `npm run build` first.',
						`resolved project root: ${projectRoot}`,
						`checked paths: ${derivedDistEntrypoint}, ${cwdDistEntrypoint}`,
					].join('\n'),
				);
			}
		},
		createEnvironment(configDirectory, overrides = {}) {
			return {
				...process.env,
				STEEL_API_KEY: apiKey || '',
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
				...overrides,
			};
		},
	};
}

export async function withCloudSession(
	harness: CloudHarness,
	options: CloudSessionOptions,
	scenario: (context: CloudSessionContext) => Promise<void> | void,
): Promise<void> {
	harness.ensureBuilt();
	if (!harness.apiKey) {
		throw new Error('Missing STEEL_API_KEY for cloud integration test.');
	}

	const configDirectory = fs.mkdtempSync(
		path.join(os.tmpdir(), options.configDirectoryPrefix),
	);
	const sessionName = `${options.sessionNamePrefix}-${Date.now()}`;
	const environment = harness.createEnvironment(
		configDirectory,
		options.environmentOverrides,
	);
	const stepProfilingEnabled = isStepProfilingEnabled();
	const stepTimings: StepTiming[] = [];
	let sessionStarted = false;

	const runCommandWithStepName = (
		stepName: string,
		arguments_: string[],
	): CommandResult => {
		const startedAt = Date.now();
		const result = runBrowserCommand(arguments_, {
			environment,
			projectRoot: harness.projectRoot,
			timeoutMs: options.commandTimeoutMs,
			attempts: options.commandAttempts,
			transientErrorPatterns: options.transientErrorPatterns,
		});

		if (stepProfilingEnabled) {
			stepTimings.push({
				stepName,
				durationMs: Date.now() - startedAt,
				status: result.status,
			});
		}

		return result;
	};

	const runCommand = (arguments_: string[]): CommandResult => {
		return runCommandWithStepName(
			`browser ${arguments_.join(' ')}`,
			arguments_,
		);
	};

	const runStep = (stepName: string, arguments_: string[]): CommandResult => {
		const result = runCommandWithStepName(stepName, arguments_);
		assertSuccessfulStep(stepName, result);
		return result;
	};

	const runFailStep = (
		stepName: string,
		arguments_: string[],
	): CommandResult => {
		const result = runCommandWithStepName(stepName, arguments_);
		assertFailedStep(stepName, result);
		return result;
	};

	const startSession = (stepName = 'browser start'): CommandResult => {
		const result = runStep(stepName, ['start', '--session', sessionName]);
		sessionStarted = true;
		return result;
	};

	const stopSession = (stepName = 'browser stop'): CommandResult => {
		return runStep(stepName, ['stop']);
	};

	try {
		await scenario({
			projectRoot: harness.projectRoot,
			configDirectory,
			sessionName,
			environment,
			runCommand,
			runStep,
			runFailStep,
			startSession,
			stopSession,
		});
	} finally {
		if (sessionStarted) {
			const stopResult = runCommand(['stop']);
			const cleanupStopBehavior = options.cleanupStopBehavior || 'assert';

			if (cleanupStopBehavior === 'assert') {
				assertSuccessfulStep('browser stop', stopResult);
			} else if (cleanupStopBehavior === 'warn' && stopResult.status !== 0) {
				console.warn(
					`Cleanup warning: browser stop failed with status ${stopResult.status}.`,
				);
			}
		}

		if (stepProfilingEnabled && stepTimings.length > 0) {
			const slowestSteps = [...stepTimings]
				.sort((left, right) => right.durationMs - left.durationMs)
				.slice(0, SLOWEST_STEP_COUNT);

			console.log(
				`[integration-step-profile] session=${sessionName} slowest ${slowestSteps.length} step(s):`,
			);
			for (const step of slowestSteps) {
				console.log(
					`- ${step.durationMs}ms | status=${step.status} | ${step.stepName}`,
				);
			}
		}

		fs.rmSync(configDirectory, {recursive: true, force: true});
	}
}
