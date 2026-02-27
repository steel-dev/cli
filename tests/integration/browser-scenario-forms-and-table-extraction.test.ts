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

describe('browser scenario forms and table extraction', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'navigates checkboxes, dropdown, and tables while extracting deterministic values',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-scenario-forms-table-'),
			);
			const sessionName = `steel-browser-scenario-forms-table-${Date.now()}`;
			const environment = {
				...process.env,
				STEEL_API_KEY: apiKey!,
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
			};

			const firstCheckboxSelector =
				'#checkboxes input[type="checkbox"]:nth-of-type(1)';
			const secondCheckboxSelector =
				'#checkboxes input[type="checkbox"]:nth-of-type(2)';
			let sessionStarted = false;

			try {
				const startResult = runBrowserCommand(
					['start', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser start', startResult);
				sessionStarted = true;

				const openCheckboxesResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/checkboxes',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open checkboxes page',
					openCheckboxesResult,
				);

				const waitCheckboxesResult = runBrowserCommand(
					['wait', '--text', 'Checkboxes', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for checkboxes heading',
					waitCheckboxesResult,
				);

				const firstCheckedInitialResult = runBrowserCommand(
					['is', 'checked', firstCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get first checkbox initial state',
					firstCheckedInitialResult,
				);
				expect(firstCheckedInitialResult.output.toLowerCase()).toContain(
					'false',
				);

				const secondCheckedInitialResult = runBrowserCommand(
					['is', 'checked', secondCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get second checkbox initial state',
					secondCheckedInitialResult,
				);
				expect(secondCheckedInitialResult.output.toLowerCase()).toContain(
					'true',
				);

				const checkFirstResult = runBrowserCommand(
					['check', firstCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser check first checkbox', checkFirstResult);

				const firstCheckedAfterCheckResult = runBrowserCommand(
					['is', 'checked', firstCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get first checkbox checked state',
					firstCheckedAfterCheckResult,
				);
				expect(firstCheckedAfterCheckResult.output.toLowerCase()).toContain(
					'true',
				);

				const uncheckSecondResult = runBrowserCommand(
					['uncheck', secondCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser uncheck second checkbox',
					uncheckSecondResult,
				);

				const secondCheckedAfterUncheckResult = runBrowserCommand(
					['is', 'checked', secondCheckboxSelector, '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get second checkbox unchecked state',
					secondCheckedAfterUncheckResult,
				);
				expect(secondCheckedAfterUncheckResult.output.toLowerCase()).toContain(
					'false',
				);

				const openDropdownResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/dropdown',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open dropdown page', openDropdownResult);

				const waitDropdownResult = runBrowserCommand(
					['wait', '--text', 'Dropdown List', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for dropdown page heading',
					waitDropdownResult,
				);

				const selectDropdownResult = runBrowserCommand(
					['select', '#dropdown', '1', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser select dropdown option 1',
					selectDropdownResult,
				);

				const getDropdownValueResult = runBrowserCommand(
					['get', 'value', '#dropdown', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get selected dropdown value',
					getDropdownValueResult,
				);
				expect(getDropdownValueResult.output).toContain('1');

				const openTablesResult = runBrowserCommand(
					[
						'open',
						'https://the-internet.herokuapp.com/tables',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open tables page', openTablesResult);

				const waitTablesResult = runBrowserCommand(
					['wait', '--text', 'Data Tables', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser wait for tables page', waitTablesResult);

				const getRowCountResult = runBrowserCommand(
					['get', 'count', '#table1 tbody tr', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get row count from table1',
					getRowCountResult,
				);
				const rowCount = Number.parseInt(getRowCountResult.output.trim(), 10);
				expect(Number.isNaN(rowCount)).toBe(false);
				expect(rowCount).toBe(4);

				const getFirstLastNameResult = runBrowserCommand(
					[
						'get',
						'text',
						'#table1 tbody tr:nth-child(1) td:nth-child(1)',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get first row last name from table1',
					getFirstLastNameResult,
				);
				expect(getFirstLastNameResult.output).toContain('Smith');

				const getFirstDueResult = runBrowserCommand(
					[
						'get',
						'text',
						'#table1 tbody tr:nth-child(1) td:nth-child(4)',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get first row due value from table1',
					getFirstDueResult,
				);
				expect(getFirstDueResult.output).toContain('$50.00');

				const getFirstActionHrefResult = runBrowserCommand(
					[
						'get',
						'attr',
						'#table1 tbody tr:nth-child(1) td:nth-child(6) a:nth-child(1)',
						'href',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get first row action href from table1',
					getFirstActionHrefResult,
				);
				expect(getFirstActionHrefResult.output).toContain('#edit');
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
