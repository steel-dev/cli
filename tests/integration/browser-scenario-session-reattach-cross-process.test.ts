import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	extractSessionId,
	withCloudSession,
} from './harness';

describe('browser scenario session reattach cross process', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'reattaches a named session across process boundaries with same session id',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-scenario-session-reattach-id-',
					sessionNamePrefix: 'steel-browser-scenario-reattach-id',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const environmentProcessA = {
						...environment,
					};
					const environmentProcessB = {
						...environment,
					};

					const startProcessAResult = startSession('process A browser start');
					assertSuccessfulStep('process A browser start', startProcessAResult);
					const processASessionId = extractSessionId(
						startProcessAResult.output,
					);

					const openLoginProcessAResult = runBrowserCommand(
						[
							'open',
							'https://the-internet.herokuapp.com/login',
							'--session',
							sessionName,
						],
						environmentProcessA,
						projectRoot,
					);
					assertSuccessfulStep(
						'process A browser open login page',
						openLoginProcessAResult,
					);

					const startProcessBResult = runBrowserCommand(
						['start', '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep('process B browser start', startProcessBResult);
					const processBSessionId = extractSessionId(
						startProcessBResult.output,
					);
					expect(processBSessionId).toBe(processASessionId);
				},
			);
		},
		90_000,
	);

	cloudTest(
		'preserves form state after reattach and allows continued login flow',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix:
						'steel-browser-scenario-session-reattach-state-',
					sessionNamePrefix: 'steel-browser-scenario-reattach-state',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const environmentProcessA = {
						...environment,
					};
					const environmentProcessB = {
						...environment,
					};

					const startProcessAResult = startSession('process A browser start');
					assertSuccessfulStep('process A browser start', startProcessAResult);

					const openLoginProcessAResult = runBrowserCommand(
						[
							'open',
							'https://the-internet.herokuapp.com/login',
							'--session',
							sessionName,
						],
						environmentProcessA,
						projectRoot,
					);
					assertSuccessfulStep(
						'process A browser open login page',
						openLoginProcessAResult,
					);

					const fillUsernameProcessAResult = runBrowserCommand(
						['fill', '#username', 'tomsmith', '--session', sessionName],
						environmentProcessA,
						projectRoot,
					);
					assertSuccessfulStep(
						'process A browser fill username',
						fillUsernameProcessAResult,
					);

					const startProcessBResult = runBrowserCommand(
						['start', '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep('process B browser start', startProcessBResult);

					const getUrlProcessBResult = runBrowserCommand(
						['get', 'url', '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser get current url',
						getUrlProcessBResult,
					);
					expect(getUrlProcessBResult.output).toContain('/login');

					const getUsernameValueProcessBResult = runBrowserCommand(
						['get', 'value', '#username', '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser get preserved username value',
						getUsernameValueProcessBResult,
					);
					expect(getUsernameValueProcessBResult.output).toContain('tomsmith');

					const fillPasswordProcessBResult = runBrowserCommand(
						[
							'fill',
							'#password',
							'SuperSecretPassword!',
							'--session',
							sessionName,
						],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser fill password',
						fillPasswordProcessBResult,
					);

					const clickLoginProcessBResult = runBrowserCommand(
						['click', "button[type='submit']", '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser click login',
						clickLoginProcessBResult,
					);

					const waitSecureAreaProcessBResult = runBrowserCommand(
						[
							'wait',
							'--text',
							'You logged into a secure area!',
							'--session',
							sessionName,
						],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser wait for secure area message',
						waitSecureAreaProcessBResult,
					);

					const getSecureUrlProcessBResult = runBrowserCommand(
						['get', 'url', '--session', sessionName],
						environmentProcessB,
						projectRoot,
					);
					assertSuccessfulStep(
						'process B browser get secure url',
						getSecureUrlProcessBResult,
					);
					expect(getSecureUrlProcessBResult.output).toContain('/secure');
				},
			);
		},
		90_000,
	);
});
