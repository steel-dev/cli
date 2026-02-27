import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

describe('browser session lifecycle expansion scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'handles tab close variants while preserving navigation context',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-session-lifecycle-tabs-',
					sessionNamePrefix: 'steel-browser-session-lifecycle-tabs',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					assertSuccessfulStep(
						'browser open windows page',
						runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/windows',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);

					assertSuccessfulStep(
						'browser open tab 1 example',
						runBrowserCommand(
							['tab', 'new', 'https://example.com', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser open tab 2 webgames',
						runBrowserCommand(
							[
								'tab',
								'new',
								'https://webgames.convergence.ai/',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);

					const tabListBeforeCloseResult = runBrowserCommand(
						['tab', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser tab list before close',
						tabListBeforeCloseResult,
					);
					expect(tabListBeforeCloseResult.output).toContain('example.com');
					expect(tabListBeforeCloseResult.output).toContain(
						'webgames.convergence.ai',
					);

					assertSuccessfulStep(
						'browser tab close by index',
						runBrowserCommand(
							['tab', 'close', '2', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					const tabListAfterIndexedCloseResult = runBrowserCommand(
						['tab', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser tab list after indexed close',
						tabListAfterIndexedCloseResult,
					);
					expect(tabListAfterIndexedCloseResult.output).not.toContain(
						'webgames.convergence.ai',
					);

					assertSuccessfulStep(
						'browser switch to tab 1',
						runBrowserCommand(
							['tab', '1', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					const getExampleTitleResult = runBrowserCommand(
						['get', 'title', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get example title',
						getExampleTitleResult,
					);
					expect(getExampleTitleResult.output).toContain('Example Domain');

					assertSuccessfulStep(
						'browser tab close current tab',
						runBrowserCommand(
							['tab', 'close', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					const tabListAfterCurrentCloseResult = runBrowserCommand(
						['tab', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser tab list after current close',
						tabListAfterCurrentCloseResult,
					);
					expect(tabListAfterCurrentCloseResult.output).not.toContain(
						'example.com',
					);
				},
			);
		},
		90_000,
	);

	cloudTest(
		'stops all sessions and allows fresh named-session startup',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-session-lifecycle-stop-all-',
					sessionNamePrefix: 'steel-browser-session-lifecycle-stop-all',
					commandAttempts: 3,
					cleanupStopBehavior: 'ignore',
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start primary session');
					assertSuccessfulStep('browser start primary session', startResult);

					const auxiliarySessionA = `steel-browser-session-lifecycle-a-${Date.now()}`;
					const auxiliarySessionB = `steel-browser-session-lifecycle-b-${Date.now()}`;

					assertSuccessfulStep(
						'browser start auxiliary session A',
						runBrowserCommand(
							['start', '--session', auxiliarySessionA],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser start auxiliary session B',
						runBrowserCommand(
							['start', '--session', auxiliarySessionB],
							environment,
							projectRoot,
						),
					);

					const sessionsBeforeStopAllResult = runBrowserCommand(
						['sessions'],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser list sessions before stop all',
						sessionsBeforeStopAllResult,
					);
					expect(sessionsBeforeStopAllResult.output).toContain(
						auxiliarySessionA,
					);
					expect(sessionsBeforeStopAllResult.output).toContain(
						auxiliarySessionB,
					);

					const stopAllResult = runBrowserCommand(
						['stop', '--all'],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser stop all sessions', stopAllResult);

					const sessionsAfterStopAllResult = runBrowserCommand(
						['sessions'],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser list sessions after stop all',
						sessionsAfterStopAllResult,
					);
					expect(sessionsAfterStopAllResult.output).not.toContain(
						auxiliarySessionA,
					);
					expect(sessionsAfterStopAllResult.output).not.toContain(
						auxiliarySessionB,
					);
					expect(sessionsAfterStopAllResult.output).not.toContain(sessionName);

					const restartedSession = `steel-browser-session-lifecycle-restart-${Date.now()}`;
					const restartResult = runBrowserCommand(
						['start', '--session', restartedSession],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser start restarted session',
						restartResult,
					);

					assertSuccessfulStep(
						'browser open example in restarted session',
						runBrowserCommand(
							['open', 'https://example.com', '--session', restartedSession],
							environment,
							projectRoot,
						),
					);

					const restartedUrlResult = runBrowserCommand(
						['get', 'url', '--session', restartedSession],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get restarted session url',
						restartedUrlResult,
					);
					expect(restartedUrlResult.output).toContain('example.com');
				},
			);
		},
		90_000,
	);
});
