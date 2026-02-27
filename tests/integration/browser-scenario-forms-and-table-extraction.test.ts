import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

describe('browser scenario forms and table extraction', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'navigates checkboxes, dropdown, and tables while extracting deterministic values',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-scenario-forms-table-',
					sessionNamePrefix: 'steel-browser-scenario-forms-table',
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const firstCheckboxSelector =
						'#checkboxes input[type="checkbox"]:nth-of-type(1)';
					const secondCheckboxSelector =
						'#checkboxes input[type="checkbox"]:nth-of-type(2)';

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

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
					assertSuccessfulStep(
						'browser check first checkbox',
						checkFirstResult,
					);

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
					expect(
						secondCheckedAfterUncheckResult.output.toLowerCase(),
					).toContain('false');

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
					assertSuccessfulStep(
						'browser open dropdown page',
						openDropdownResult,
					);

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
					assertSuccessfulStep(
						'browser wait for tables page',
						waitTablesResult,
					);

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
				},
			);
		},
		90_000,
	);
});
