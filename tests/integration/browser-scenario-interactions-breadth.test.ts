import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

describe('browser interactions breadth scenario', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'covers keyboard commands with value/html/attr/style/box extraction',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-interactions-breadth-keyboard-',
					sessionNamePrefix: 'steel-browser-interactions-keyboard',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					assertSuccessfulStep(
						'browser open inputs page',
						runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/inputs',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser focus input',
						runBrowserCommand(
							['focus', 'input', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser type into input',
						runBrowserCommand(
							['type', 'input', '12345', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					const getInputValueResult = runBrowserCommand(
						['get', 'value', 'input', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser get typed value', getInputValueResult);
					expect(getInputValueResult.output).toContain('12345');

					assertSuccessfulStep(
						'browser open key presses page',
						runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/key_presses',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser focus key target input',
						runBrowserCommand(
							['focus', '#target', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					assertSuccessfulStep(
						'browser press enter',
						runBrowserCommand(
							['press', 'Enter', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					const keyResultAfterEnter = runBrowserCommand(
						['get', 'text', '#result', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get key result after enter',
						keyResultAfterEnter,
					);

					if (!keyResultAfterEnter.output.toUpperCase().includes('ENTER')) {
						assertSuccessfulStep(
							'browser click key target input',
							runBrowserCommand(
								['click', '#target', '--session', sessionName],
								environment,
								projectRoot,
							),
						);
						assertSuccessfulStep(
							'browser press fallback key A',
							runBrowserCommand(
								['press', 'A', '--session', sessionName],
								environment,
								projectRoot,
							),
						);
					}

					assertSuccessfulStep(
						'browser keydown shift',
						runBrowserCommand(
							['keydown', 'Shift', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser keyup shift',
						runBrowserCommand(
							['keyup', 'Shift', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					const getResultHtml = runBrowserCommand(
						['get', 'html', '#result', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser get result html', getResultHtml);
					expect(getResultHtml.output).toContain('You entered');

					const getTargetIdAttr = runBrowserCommand(
						['get', 'attr', '#target', 'id', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get target id attribute',
						getTargetIdAttr,
					);
					expect(getTargetIdAttr.output).toContain('target');

					const getTargetStyles = runBrowserCommand(
						['get', 'styles', '#target', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser get target styles', getTargetStyles);
					expect(getTargetStyles.output.trim().length).toBeGreaterThan(0);

					const getTargetBox = runBrowserCommand(
						['get', 'box', '#target', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser get target box', getTargetBox);
					expect(getTargetBox.output.toLowerCase()).toMatch(/x|width|height/);
				},
			);
		},
		90_000,
	);

	cloudTest(
		'covers first/last/nth find actions plus hover and scroll controls',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-interactions-breadth-pointer-',
					sessionNamePrefix: 'steel-browser-interactions-pointer',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					assertSuccessfulStep(
						'browser open add remove elements page',
						runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/add_remove_elements/',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);

					for (let count = 0; count < 3; count += 1) {
						assertSuccessfulStep(
							`browser add element ${count + 1}`,
							runBrowserCommand(
								[
									'click',
									'button[onclick="addElement()"]',
									'--session',
									sessionName,
								],
								environment,
								projectRoot,
							),
						);
					}

					const countAfterAddResult = runBrowserCommand(
						['get', 'count', '.added-manually', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser count added elements after add',
						countAfterAddResult,
					);
					expect(Number.parseInt(countAfterAddResult.output.trim(), 10)).toBe(
						3,
					);

					assertSuccessfulStep(
						'browser find first delete click',
						runBrowserCommand(
							[
								'find',
								'first',
								'.added-manually',
								'click',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser find last delete click',
						runBrowserCommand(
							[
								'find',
								'last',
								'.added-manually',
								'click',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser find nth delete click',
						runBrowserCommand(
							[
								'find',
								'nth',
								'0',
								'.added-manually',
								'click',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);

					const countAfterDeleteResult = runBrowserCommand(
						['get', 'count', '.added-manually', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser count added elements after deletes',
						countAfterDeleteResult,
					);
					expect(
						Number.parseInt(countAfterDeleteResult.output.trim(), 10),
					).toBe(0);

					assertSuccessfulStep(
						'browser open hovers page',
						runBrowserCommand(
							[
								'open',
								'https://the-internet.herokuapp.com/hovers',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser hover first avatar',
						runBrowserCommand(
							['hover', '.figure:nth-of-type(1) img', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser wait for hover details',
						runBrowserCommand(
							['wait', '--text', 'name: user1', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

					assertSuccessfulStep(
						'browser open webgames',
						runBrowserCommand(
							[
								'open',
								'https://webgames.convergence.ai/',
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser wait for webgames homepage',
						runBrowserCommand(
							['wait', '--text', 'WebGames', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser scroll homepage down',
						runBrowserCommand(
							['scroll', 'down', '500', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser mouse wheel scroll',
						runBrowserCommand(
							['mouse', 'wheel', '300', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser scroll into view challenge card',
						runBrowserCommand(
							[
								'scrollintoview',
								"a[href='/shopping-challenge']",
								'--session',
								sessionName,
							],
							environment,
							projectRoot,
						),
					);
				},
			);
		},
		90_000,
	);
});
