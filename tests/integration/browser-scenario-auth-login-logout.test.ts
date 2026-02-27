import {createCloudHarness, withCloudSession} from './harness';

describe('browser auth scenario login/logout', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'logs in and logs out successfully on practicetestautomation.com',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-auth-login-logout-',
					sessionNamePrefix: 'steel-browser-auth',
				},
				({sessionName, startSession, runStep}) => {
					startSession();

					runStep('browser open login page', [
						'open',
						'https://practicetestautomation.com/practice-test-login/',
						'--session',
						sessionName,
					]);

					const loginSnapshotResult = runStep('browser snapshot login page', [
						'snapshot',
						'-i',
						'--session',
						sessionName,
					]);
					expect(loginSnapshotResult.output).toContain('Username');
					expect(loginSnapshotResult.output).toContain('Password');
					expect(loginSnapshotResult.output).toMatch(/ref=e\d+/);

					runStep('browser fill username', [
						'fill',
						'#username',
						'student',
						'--session',
						sessionName,
					]);
					runStep('browser fill password', [
						'fill',
						'#password',
						'Password123',
						'--session',
						sessionName,
					]);
					runStep('browser click login', [
						'click',
						'#submit',
						'--session',
						sessionName,
					]);

					runStep('browser wait for secure area success text', [
						'wait',
						'--text',
						'Logged In Successfully',
						'--session',
						sessionName,
					]);

					const getSecureUrlResult = runStep('browser get secure url', [
						'get',
						'url',
						'--session',
						sessionName,
					]);
					expect(getSecureUrlResult.output).toContain(
						'/logged-in-successfully/',
					);

					const getSecureFlashResult = runStep(
						'browser get secure flash text',
						['get', 'text', '.post-title', '--session', sessionName],
					);
					expect(getSecureFlashResult.output).toContain(
						'Logged In Successfully',
					);

					runStep('browser click logout', [
						'click',
						'.wp-block-button__link',
						'--session',
						sessionName,
					]);

					runStep('browser wait for login url after logout', [
						'wait',
						'--url',
						'**/practice-test-login/',
						'--session',
						sessionName,
					]);

					const getLoggedOutUrlResult = runStep('browser get logged-out url', [
						'get',
						'url',
						'--session',
						sessionName,
					]);
					expect(getLoggedOutUrlResult.output).toContain(
						'/practice-test-login/',
					);

					const getLoggedOutFlashResult = runStep(
						'browser get logged-out flash text',
						['get', 'title', '--session', sessionName],
					);
					expect(getLoggedOutFlashResult.output).toContain('Test Login');
				},
			);
		},
		90_000,
	);

	cloudTest(
		'fails login with bad credentials and exits non-zero on protected action',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-auth-bad-creds-',
					sessionNamePrefix: 'steel-browser-auth-bad-creds',
				},
				({sessionName, startSession, runStep, runFailStep}) => {
					startSession();

					runStep('browser open login page', [
						'open',
						'https://practicetestautomation.com/practice-test-login/',
						'--session',
						sessionName,
					]);
					runStep('browser fill username', [
						'fill',
						'#username',
						'student',
						'--session',
						sessionName,
					]);
					runStep('browser fill password', [
						'fill',
						'#password',
						'wrong-password',
						'--session',
						sessionName,
					]);
					runStep('browser click login', [
						'click',
						'#submit',
						'--session',
						sessionName,
					]);

					const getInvalidFlashResult = runStep(
						'browser get invalid-credentials flash text',
						['get', 'text', '#error', '--session', sessionName],
					);
					expect(getInvalidFlashResult.output).toContain(
						'Your password is invalid!',
					);

					runFailStep('browser click logout after bad credentials', [
						'click',
						'.wp-block-button__link',
						'--session',
						sessionName,
					]);
				},
			);
		},
		90_000,
	);
});
