import {createCloudHarness, withCloudSession} from './harness';

describe('browser failure and recovery contracts scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'returns non-zero for invalid selectors with stable failure output',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-failure-invalid-selector-',
					sessionNamePrefix: 'steel-browser-failure-invalid-selector',
					commandAttempts: 3,
				},
				({sessionName, startSession, runStep, runFailStep}) => {
					startSession();

					runStep('browser open example for invalid selector check', [
						'open',
						'https://example.com',
						'--session',
						sessionName,
					]);

					const invalidSelectorResult = runFailStep(
						'browser click invalid selector',
						['click', '#does-not-exist', '--session', sessionName],
					);
					expect(invalidSelectorResult.output.toLowerCase()).toMatch(
						/not found|waiting for|timeout|timed out|failed/,
					);
				},
			);
		},
		90_000,
	);

	cloudTest(
		'recovers from command failure and continues in the same session',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-failure-same-session-recover-',
					sessionNamePrefix: 'steel-browser-failure-same-session-recover',
					commandAttempts: 3,
				},
				({sessionName, startSession, runStep, runFailStep}) => {
					startSession();

					runStep('browser open login page for recovery test', [
						'open',
						'https://the-internet.herokuapp.com/login',
						'--session',
						sessionName,
					]);

					runFailStep('browser fail with invalid login selector', [
						'click',
						'#missing-login-button',
						'--session',
						sessionName,
					]);

					runStep('browser fill username after failure', [
						'fill',
						'#username',
						'tomsmith',
						'--session',
						sessionName,
					]);

					const usernameValueResult = runStep(
						'browser get username value after failure recovery',
						['get', 'value', '#username', '--session', sessionName],
					);
					expect(usernameValueResult.output).toContain('tomsmith');
				},
			);
		},
		90_000,
	);

	cloudTest(
		'returns to a blank page after stop and recovers after restart',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-failure-stop-restart-',
					sessionNamePrefix: 'steel-browser-failure-stop-restart',
					commandAttempts: 3,
					cleanupStopBehavior: 'ignore',
				},
				({sessionName, startSession, stopSession, runStep}) => {
					startSession();

					runStep('browser open page before stop', [
						'open',
						'https://example.com',
						'--session',
						sessionName,
					]);

					stopSession('browser stop before failure check');

					const commandAfterStopResult = runStep('browser get url after stop', [
						'get',
						'url',
						'--session',
						sessionName,
					]);
					expect(commandAfterStopResult.output.toLowerCase()).toContain(
						'about:blank',
					);

					runStep('browser restart same named session', [
						'start',
						'--session',
						sessionName,
					]);

					runStep('browser open page after restart', [
						'open',
						'https://example.com',
						'--session',
						sessionName,
					]);

					const urlAfterRestartResult = runStep(
						'browser get url after restart',
						['get', 'url', '--session', sessionName],
					);
					expect(urlAfterRestartResult.output).toContain('example.com');
				},
			);
		},
		90_000,
	);
});
