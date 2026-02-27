import {createCloudHarness, withCloudSession} from './harness';

describe('browser cloud login workflow e2e', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'executes a richer login workflow with click/fill/scroll/snapshot commands',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-cloud-login-e2e-',
					sessionNamePrefix: 'steel-browser-login-e2e',
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
					expect(loginSnapshotResult.output).toContain('Submit');
					expect(loginSnapshotResult.output).toMatch(/ref=e\d+/);

					runStep('browser scroll down', [
						'scroll',
						'down',
						'300',
						'--session',
						sessionName,
					]);
					runStep('browser scroll up', [
						'scroll',
						'up',
						'200',
						'--session',
						sessionName,
					]);
					runStep('browser click username', [
						'click',
						'#username',
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
					runStep('browser click password', [
						'click',
						'#password',
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

					runStep('browser wait for login success', [
						'wait',
						'--text',
						'Logged In Successfully',
						'--session',
						sessionName,
					]);

					const getUrlResult = runStep('browser get url', [
						'get',
						'url',
						'--session',
						sessionName,
					]);
					expect(getUrlResult.output).toContain('/logged-in-successfully/');

					const getPageHeaderResult = runStep(
						'browser get logged-in page title text',
						['get', 'text', '.post-title', '--session', sessionName],
					);
					expect(getPageHeaderResult.output).toContain(
						'Logged In Successfully',
					);

					const getLinkCountResult = runStep('browser get link count', [
						'get',
						'count',
						'a',
						'--session',
						sessionName,
					]);
					const linkCount = Number.parseInt(
						getLinkCountResult.output.trim(),
						10,
					);
					expect(Number.isNaN(linkCount)).toBe(false);
					expect(linkCount).toBeGreaterThanOrEqual(1);

					const secureSnapshotResult = runStep(
						'browser snapshot logged-in page',
						['snapshot', '-i', '--session', sessionName],
					);
					expect(secureSnapshotResult.output).toContain('Log out');
					expect(secureSnapshotResult.output).toMatch(/ref=e\d+/);

					runStep('browser click logout', [
						'click',
						'.wp-block-button__link',
						'--session',
						sessionName,
					]);

					runStep('browser wait for login page after logout', [
						'wait',
						'--text',
						'Test login',
						'--session',
						sessionName,
					]);

					const getLoggedOutUrlResult = runStep(
						'browser get url after logout',
						['get', 'url', '--session', sessionName],
					);
					expect(getLoggedOutUrlResult.output).toContain(
						'/practice-test-login/',
					);
				},
			);
		},
		90_000,
	);
});
