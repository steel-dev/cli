import {createCloudHarness, withCloudSession} from './harness';

describe('browser cloud e2e', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'controls a real cloud browser session end-to-end',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-cloud-e2e-',
					sessionNamePrefix: 'steel-browser-e2e',
				},
				({sessionName, startSession, runStep}) => {
					startSession();

					runStep('browser open', [
						'open',
						'https://example.com',
						'--session',
						sessionName,
					]);

					const getTitleResult = runStep('browser get title', [
						'get',
						'title',
						'--session',
						sessionName,
					]);
					expect(getTitleResult.output).toContain('Example Domain');

					const snapshotResult = runStep('browser snapshot -i', [
						'snapshot',
						'-i',
						'--session',
						sessionName,
					]);
					expect(snapshotResult.output).toMatch(/ref=e\d+/);
				},
			);
		},
		90_000,
	);
});
