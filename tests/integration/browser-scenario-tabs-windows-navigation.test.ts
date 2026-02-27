import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

describe('browser tabs windows navigation scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'handles tab/window controls with navigation state checks',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-tabs-windows-nav-',
					sessionNamePrefix: 'steel-browser-tabs-windows-nav',
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

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
					assertSuccessfulStep(
						'browser reopen windows page',
						openWindowsResult,
					);

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
				},
			);
		},
		90_000,
	);
});
