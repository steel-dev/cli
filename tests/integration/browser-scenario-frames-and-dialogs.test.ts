import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

describe('browser frames and dialogs scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'handles iframe commands and javascript alert/confirm/prompt dialogs',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-frames-dialogs-',
					sessionNamePrefix: 'steel-browser-frames-dialogs',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

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
					assertSuccessfulStep(
						'browser open iframe page',
						openIframePageResult,
					);

					const switchToIframeResult = runBrowserCommand(
						['frame', '#mce_0_ifr', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser switch to iframe',
						switchToIframeResult,
					);

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
					assertSuccessfulStep(
						'browser dialog accept alert',
						acceptAlertResult,
					);

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
						[
							'click',
							"button[onclick='jsConfirm()']",
							'--session',
							sessionName,
						],
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
				},
			);
		},
		90_000,
	);
});
