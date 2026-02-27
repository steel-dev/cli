import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	extractRefForText,
	withCloudSession,
} from './harness';

describe('browser semantic locators + refs scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'mixes snapshot refs with semantic role/text/label/placeholder locators',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-semantic-plus-refs-',
					sessionNamePrefix: 'steel-browser-semantic-plus-refs',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					const openPracticeLoginResult = runBrowserCommand(
						[
							'open',
							'https://practicetestautomation.com/practice-test-login/',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser open practice login page',
						openPracticeLoginResult,
					);

					const snapshotResult = runBrowserCommand(
						['snapshot', '-i', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser snapshot -i', snapshotResult);
					expect(snapshotResult.output).toContain('Username');
					expect(snapshotResult.output).toMatch(/ref=e\d+/);

					const usernameRef = extractRefForText(
						snapshotResult.output,
						'Username',
					);

					const fillUsernameByRefResult = runBrowserCommand(
						['fill', usernameRef, 'student', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser fill username via ref',
						fillUsernameByRefResult,
					);

					const fillPasswordByLabelResult = runBrowserCommand(
						[
							'find',
							'label',
							'Password',
							'fill',
							'Password123',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser find label password fill',
						fillPasswordByLabelResult,
					);

					const clickSubmitByRoleResult = runBrowserCommand(
						[
							'find',
							'role',
							'button',
							'click',
							'--name',
							'Submit',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser find role button click submit',
						clickSubmitByRoleResult,
					);

					const waitLoggedInResult = runBrowserCommand(
						[
							'wait',
							'--text',
							'Logged In Successfully',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser wait logged-in text',
						waitLoggedInResult,
					);

					const clickLogoutByTextResult = runBrowserCommand(
						[
							'find',
							'text',
							'Log out',
							'click',
							'--exact',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser find text logout click',
						clickLogoutByTextResult,
					);

					const waitLoginUrlResult = runBrowserCommand(
						[
							'wait',
							'--url',
							'**/practice-test-login/',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser wait login url after logout',
						waitLoginUrlResult,
					);

					const openSauceDemoResult = runBrowserCommand(
						['open', 'https://www.saucedemo.com/', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser open saucedemo', openSauceDemoResult);

					const fillByPlaceholderResult = runBrowserCommand(
						[
							'find',
							'placeholder',
							'Username',
							'fill',
							'standard_user',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser find placeholder username fill',
						fillByPlaceholderResult,
					);

					const getSauceUsernameValueResult = runBrowserCommand(
						['get', 'value', '#user-name', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get saucedemo username value',
						getSauceUsernameValueResult,
					);
					expect(getSauceUsernameValueResult.output).toContain('standard_user');
				},
			);
		},
		90_000,
	);
});
