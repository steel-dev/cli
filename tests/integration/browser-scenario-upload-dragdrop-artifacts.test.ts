import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

function assertNonEmptyFile(filePath: string): void {
	expect(fs.existsSync(filePath)).toBe(true);
	const stats = fs.statSync(filePath);
	expect(stats.size).toBeGreaterThan(0);
}

describe('browser upload dragdrop artifacts scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'generates screenshot and pdf artifacts to explicit output paths',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-artifacts-generate-config-',
					sessionNamePrefix: 'steel-browser-artifacts-generate',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const artifactDirectory = fs.mkdtempSync(
						path.join(os.tmpdir(), 'steel-browser-artifacts-generate-'),
					);
					const screenshotPath = path.join(artifactDirectory, 'example.png');
					const fullScreenshotPath = path.join(
						artifactDirectory,
						'example-full.png',
					);
					const pdfPath = path.join(artifactDirectory, 'example.pdf');

					try {
						const startResult = startSession('browser start');
						assertSuccessfulStep('browser start', startResult);

						const openExampleResult = runBrowserCommand(
							['open', 'https://example.com', '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('browser open example', openExampleResult);

						const screenshotResult = runBrowserCommand(
							['screenshot', screenshotPath, '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser screenshot to explicit path',
							screenshotResult,
						);
						assertNonEmptyFile(screenshotPath);

						const fullScreenshotResult = runBrowserCommand(
							[
								'screenshot',
								fullScreenshotPath,
								'--full',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser full screenshot to explicit path',
							fullScreenshotResult,
						);
						assertNonEmptyFile(fullScreenshotPath);

						const pdfResult = runBrowserCommand(
							['pdf', pdfPath, '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('browser pdf to explicit path', pdfResult);
						assertNonEmptyFile(pdfPath);
					} finally {
						fs.rmSync(artifactDirectory, {recursive: true, force: true});
					}
				},
			);
		},
		90_000,
	);

	cloudTest(
		'uploads a file and verifies uploaded filename output',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-upload-file-config-',
					sessionNamePrefix: 'steel-browser-upload-file',
					commandAttempts: 5,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const artifactDirectory = fs.mkdtempSync(
						path.join(os.tmpdir(), 'steel-browser-upload-file-'),
					);
					const uploadFilePath = path.join(
						artifactDirectory,
						'upload-sample.txt',
					);
					fs.writeFileSync(
						uploadFilePath,
						'steel upload artifact sample\n',
						'utf-8',
					);

					try {
						const startResult = startSession('browser start');
						assertSuccessfulStep('browser start', startResult);

						const openUploadPageResult = runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/upload',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser open upload page',
							openUploadPageResult,
						);

						const waitUploadInputResult = runBrowserCommand(
							['wait', '#file-upload', '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser wait for upload input',
							waitUploadInputResult,
						);

						const uploadFileResult = runBrowserCommand(
							[
								'upload',
								'#file-upload',
								uploadFilePath,
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('browser upload file', uploadFileResult);

						const clickUploadSubmitResult = runBrowserCommand(
							['click', '#file-submit', '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser click upload submit',
							clickUploadSubmitResult,
						);

						const waitUploadedResult = runBrowserCommand(
							['wait', '--text', 'File Uploaded!', '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser wait uploaded confirmation',
							waitUploadedResult,
						);

						const getUploadedFileNameResult = runBrowserCommand(
							['get', 'text', '#uploaded-files', '--session', sessionName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep(
							'browser get uploaded file name text',
							getUploadedFileNameResult,
						);
						expect(getUploadedFileNameResult.output).toContain(
							path.basename(uploadFilePath),
						);
					} finally {
						fs.rmSync(artifactDirectory, {recursive: true, force: true});
					}
				},
			);
		},
		90_000,
	);

	cloudTest(
		'performs drag-drop and mouse primitives on drag-and-drop page',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-dragdrop-mouse-config-',
					sessionNamePrefix: 'steel-browser-dragdrop-mouse',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					const openDragDropPageResult = runBrowserCommand(
						[
							'open',
							'https://the-internet.herokuapp.com/drag_and_drop',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser open dragdrop page',
						openDragDropPageResult,
					);

					const getColumnABeforeDragResult = runBrowserCommand(
						['get', 'text', '#column-a header', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get column-a header before drag',
						getColumnABeforeDragResult,
					);
					expect(getColumnABeforeDragResult.output.trim()).toBe('A');

					const getColumnBBeforeDragResult = runBrowserCommand(
						['get', 'text', '#column-b header', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get column-b header before drag',
						getColumnBBeforeDragResult,
					);
					expect(getColumnBBeforeDragResult.output.trim()).toBe('B');

					const dragResult = runBrowserCommand(
						['drag', '#column-a', '#column-b', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser drag column-a to column-b', dragResult);

					const getColumnAAfterDragResult = runBrowserCommand(
						['get', 'text', '#column-a header', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get column-a header after drag',
						getColumnAAfterDragResult,
					);
					expect(getColumnAAfterDragResult.output.trim()).toBe('B');

					const getColumnBAfterDragResult = runBrowserCommand(
						['get', 'text', '#column-b header', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get column-b header after drag',
						getColumnBAfterDragResult,
					);
					expect(getColumnBAfterDragResult.output.trim()).toBe('A');

					const mouseMoveResult = runBrowserCommand(
						['mouse', 'move', '200', '200', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser mouse move', mouseMoveResult);

					const mouseDownResult = runBrowserCommand(
						['mouse', 'down', 'left', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser mouse down', mouseDownResult);

					const mouseUpResult = runBrowserCommand(
						['mouse', 'up', 'left', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser mouse up', mouseUpResult);
				},
			);
		},
		90_000,
	);
});
