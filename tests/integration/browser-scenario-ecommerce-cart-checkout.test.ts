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

describe('browser scenario ecommerce cart checkout', () => {
	const testDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(testDirectory, '../..');
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	const apiKey = process.env.STEEL_API_KEY?.trim();
	const cloudTest = apiKey ? test : test.skip;

	cloudTest(
		'executes demoblaze cart operations and checkout flow',
		() => {
			if (!fs.existsSync(distEntrypoint)) {
				throw new Error('dist/steel.js is missing. Run `npm run build` first.');
			}

			const configDirectory = fs.mkdtempSync(
				path.join(os.tmpdir(), 'steel-browser-scenario-ecommerce-'),
			);
			const sessionName = `steel-browser-scenario-ecommerce-${Date.now()}`;
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

				const openStoreResult = runBrowserCommand(
					['open', 'https://www.demoblaze.com/', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open demoblaze', openStoreResult);

				const waitStoreHomeResult = runBrowserCommand(
					['wait', '--text', 'PRODUCT STORE', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for store home',
					waitStoreHomeResult,
				);

				const openFirstProductResult = runBrowserCommand(
					[
						'find',
						'first',
						'a[href="prod.html?idp_=1"]',
						'click',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open first product',
					openFirstProductResult,
				);

				const waitFirstProductResult = runBrowserCommand(
					['wait', '--text', 'Samsung galaxy s6', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for first product',
					waitFirstProductResult,
				);

				const addFirstProductResult = runBrowserCommand(
					['click', '.btn.btn-success.btn-lg', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser add first product to cart',
					addFirstProductResult,
				);

				const acceptFirstDialogResult = runBrowserCommand(
					['dialog', 'accept', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser accept add-to-cart dialog for first product',
					acceptFirstDialogResult,
				);

				const returnHomeAfterFirstAddResult = runBrowserCommand(
					['open', 'https://www.demoblaze.com/', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser return to home after first add',
					returnHomeAfterFirstAddResult,
				);

				const waitHomeAfterFirstAddResult = runBrowserCommand(
					['wait', '--text', 'PRODUCT STORE', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for home after first add',
					waitHomeAfterFirstAddResult,
				);

				const openSecondProductResult = runBrowserCommand(
					[
						'open',
						'https://www.demoblaze.com/prod.html?idp_=2',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser open second product',
					openSecondProductResult,
				);

				const waitSecondProductResult = runBrowserCommand(
					['wait', '--text', 'Nokia lumia 1520', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for second product',
					waitSecondProductResult,
				);

				const addSecondProductResult = runBrowserCommand(
					['click', '.btn.btn-success.btn-lg', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser add second product to cart',
					addSecondProductResult,
				);

				const acceptSecondDialogResult = runBrowserCommand(
					['dialog', 'accept', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser accept add-to-cart dialog for second product',
					acceptSecondDialogResult,
				);

				const openCartResult = runBrowserCommand(
					[
						'open',
						'https://www.demoblaze.com/cart.html',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser open cart', openCartResult);

				const waitCartPageResult = runBrowserCommand(
					['wait', '--text', 'Products', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser wait for cart page', waitCartPageResult);

				const waitCartRowsResult = runBrowserCommand(
					['wait', '1000', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser wait for cart rows', waitCartRowsResult);

				const getCartItemCountResult = runBrowserCommand(
					['get', 'count', '#tbodyid > tr', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get cart item count',
					getCartItemCountResult,
				);
				expect(Number.parseInt(getCartItemCountResult.output.trim(), 10)).toBe(
					2,
				);

				const getCartItemNameResult = runBrowserCommand(
					['get', 'text', '#tbodyid', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get cart item name',
					getCartItemNameResult,
				);
				expect(getCartItemNameResult.output).toContain('Samsung galaxy s6');
				expect(getCartItemNameResult.output).toContain('Nokia lumia 1520');

				const deleteFirstRowResult = runBrowserCommand(
					['click', '#tbodyid > tr:nth-child(1) a', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser remove first cart row',
					deleteFirstRowResult,
				);

				const waitDeleteResult = runBrowserCommand(
					['wait', '1500', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser wait for cart delete', waitDeleteResult);

				const getCartItemCountAfterDeleteResult = runBrowserCommand(
					['get', 'count', '#tbodyid > tr', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get cart item count after delete',
					getCartItemCountAfterDeleteResult,
				);
				expect(
					Number.parseInt(getCartItemCountAfterDeleteResult.output.trim(), 10),
				).toBe(1);

				const checkoutVisibleResult = runBrowserCommand(
					[
						'is',
						'visible',
						'button[data-target="#orderModal"]',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser check place order visibility',
					checkoutVisibleResult,
				);
				expect(checkoutVisibleResult.output.toLowerCase()).toContain('true');

				const clickCheckoutResult = runBrowserCommand(
					[
						'click',
						'button[data-target="#orderModal"]',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser click place order', clickCheckoutResult);

				const waitCheckoutInfoResult = runBrowserCommand(
					['wait', '#name', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait for checkout information modal',
					waitCheckoutInfoResult,
				);

				const fillFirstNameResult = runBrowserCommand(
					['fill', '#name', 'Steel', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order name', fillFirstNameResult);

				const fillLastNameResult = runBrowserCommand(
					['fill', '#country', 'US', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order country', fillLastNameResult);

				const fillPostalCodeResult = runBrowserCommand(
					['fill', '#city', 'NYC', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order city', fillPostalCodeResult);

				const fillCardResult = runBrowserCommand(
					['fill', '#card', '4242424242424242', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order card', fillCardResult);

				const fillMonthResult = runBrowserCommand(
					['fill', '#month', '12', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order month', fillMonthResult);

				const fillYearResult = runBrowserCommand(
					['fill', '#year', '2030', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep('browser fill order year', fillYearResult);

				const finishCheckoutResult = runBrowserCommand(
					[
						'click',
						'button[onclick="purchaseOrder()"]',
						'--session',
						sessionName,
					],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser attempt purchase order',
					finishCheckoutResult,
				);

				const waitPostPurchaseResult = runBrowserCommand(
					['wait', '1500', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser wait after purchase attempt',
					waitPostPurchaseResult,
				);

				const getCompleteUrlResult = runBrowserCommand(
					['get', 'url', '--session', sessionName],
					environment,
					projectRoot,
				);
				assertSuccessfulStep(
					'browser get cart url after purchase',
					getCompleteUrlResult,
				);
				expect(getCompleteUrlResult.output).toContain('/cart.html');
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
			}
		},
		240_000,
	);
});
