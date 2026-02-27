import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

type CommandResult = {
	status: number;
	stdout: string;
	stderr: string;
	output: string;
};

const COMMAND_TIMEOUT_MS = 90_000;

function runBrowserCommand(
	arguments_: string[],
	environment: NodeJS.ProcessEnv,
	projectRoot: string,
): CommandResult {
	const result = spawnSync(
		process.execPath,
		['dist/steel.js', 'browser', ...arguments_],
		{
			cwd: projectRoot,
			env: environment,
			encoding: 'utf-8',
			timeout: COMMAND_TIMEOUT_MS,
			killSignal: 'SIGKILL',
		},
	);

	const stdout = result.stdout || '';
	const stderrParts = [result.stderr || ''];

	if (result.error) {
		stderrParts.push(`spawn error: ${result.error.message}`);
	}

	if (result.signal) {
		stderrParts.push(`terminated by signal: ${result.signal}`);
	}

	const stderr = stderrParts.filter(Boolean).join('\n');
	const output = [stdout, stderr].filter(Boolean).join('\n');

	return {
		status: result.status ?? 1,
		stdout,
		stderr,
		output,
	};
}

function assertSuccessfulStep(stepName: string, result: CommandResult): void {
	if (result.status === 0) {
		return;
	}

	throw new Error(
		[
			`${stepName} failed with exit code ${result.status}.`,
			result.stdout.trim() ? `stdout:\n${result.stdout.trim()}` : null,
			result.stderr.trim() ? `stderr:\n${result.stderr.trim()}` : null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

function assertNonEmptyFile(filePath: string): void {
	expect(fs.existsSync(filePath)).toBe(true);
	const stats = fs.statSync(filePath);
	expect(stats.size).toBeGreaterThan(0);
}

describe('browser upload dragdrop artifacts scenario', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'uploads file, performs drag-drop, and generates screenshot/pdf artifacts',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-upload-dragdrop-config-'),
			);
			const sessionName = `steel-browser-upload-dragdrop-${Date.now()}`;
			const artifactDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-upload-dragdrop-artifacts-'),
			);
			const uploadFilePath = path.join(artifactDirectory, 'upload-sample.txt');
			fs.writeFileSync(
				uploadFilePath,
				'steel upload artifact sample\n',
				'utf-8',
			);

			const screenshotPath = path.join(artifactDirectory, 'example.png');
			const fullScreenshotPath = path.join(
				artifactDirectory,
				'example-full.png',
			);
			const pdfPath = path.join(artifactDirectory, 'example.pdf');

			const environment = {
				...process.env,
				STEEL_API_KEY: apiKey!,
				STEEL_CONFIG_DIR: configDirectory,
				STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				FORCE_COLOR: '0',
				NODE_NO_WARNINGS: '1',
			};

			let sessionStarted = false;

			try {
				const startResult = runBrowserCommand(
					['start', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser start', startResult);
				sessionStarted = true;

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
				assertSuccessfulStep('browser open upload page', openUploadPageResult);

				const uploadFileResult = runBrowserCommand(
					['upload', '#file-upload', uploadFilePath, '--session', sessionName],
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
			} finally {
				if (sessionStarted) {
					const stopResult = runBrowserCommand(
						['stop'],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser stop', stopResult);
				}

				fs.rmSync(configDirectory, {recursive: true, force: true});
				fs.rmSync(artifactDirectory, {recursive: true, force: true});
			}
		},
		240_000,
	);
});
