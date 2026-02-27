import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	withCloudSession,
} from './harness';

type ScenarioContext = {
	sessionName: string;
	environment: NodeJS.ProcessEnv;
	projectRoot: string;
	runBrowserCommand: ReturnType<typeof createLegacyRunBrowserCommand>;
};

function addProductToCart(
	context: ScenarioContext,
	productUrl: string,
	productName: string,
	stepPrefix: string,
): void {
	const {sessionName, environment, projectRoot, runBrowserCommand} = context;

	const openProductResult = runBrowserCommand(
		['open', productUrl, '--session', sessionName],
		environment,
		projectRoot,
	);
	assertSuccessfulStep(`${stepPrefix} open product`, openProductResult);

	const waitProductResult = runBrowserCommand(
		['wait', '--text', productName, '--session', sessionName],
		environment,
		projectRoot,
	);
	assertSuccessfulStep(`${stepPrefix} wait for product`, waitProductResult);

	const addProductResult = runBrowserCommand(
		['click', '.btn.btn-success.btn-lg', '--session', sessionName],
		environment,
		projectRoot,
	);
	assertSuccessfulStep(`${stepPrefix} add product to cart`, addProductResult);

	const acceptDialogResult = runBrowserCommand(
		['dialog', 'accept', '--session', sessionName],
		environment,
		projectRoot,
	);
	assertSuccessfulStep(
		`${stepPrefix} accept add-to-cart dialog`,
		acceptDialogResult,
	);
}

function openCart(context: ScenarioContext): void {
	const {sessionName, environment, projectRoot, runBrowserCommand} = context;

	const openCartResult = runBrowserCommand(
		['open', 'https://www.demoblaze.com/cart.html', '--session', sessionName],
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
}

describe('browser scenario ecommerce cart checkout', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'adds products to cart and verifies line items',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-scenario-ecommerce-add-items-',
					sessionNamePrefix: 'steel-browser-scenario-ecommerce-add-items',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const context = {
						sessionName,
						environment,
						projectRoot,
						runBrowserCommand,
					};

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					addProductToCart(
						context,
						'https://www.demoblaze.com/prod.html?idp_=1',
						'Samsung galaxy s6',
						'browser first product',
					);
					addProductToCart(
						context,
						'https://www.demoblaze.com/prod.html?idp_=2',
						'Nokia lumia 1520',
						'browser second product',
					);

					openCart(context);

					const waitCartRowsResult = runBrowserCommand(
						['wait', '1200', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser wait for cart rows',
						waitCartRowsResult,
					);

					const getCartItemNameResult = runBrowserCommand(
						['get', 'text', '#tbodyid', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep('browser get cart text', getCartItemNameResult);
					expect(getCartItemNameResult.output).toContain('Samsung galaxy s6');
					expect(getCartItemNameResult.output).toContain('Nokia lumia 1520');
				},
			);
		},
		90_000,
	);

	cloudTest(
		'removes an item from cart and keeps checkout available',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix:
						'steel-browser-scenario-ecommerce-remove-item-',
					sessionNamePrefix: 'steel-browser-scenario-ecommerce-remove-item',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const context = {
						sessionName,
						environment,
						projectRoot,
						runBrowserCommand,
					};

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					addProductToCart(
						context,
						'https://www.demoblaze.com/prod.html?idp_=1',
						'Samsung galaxy s6',
						'browser first product',
					);
					addProductToCart(
						context,
						'https://www.demoblaze.com/prod.html?idp_=2',
						'Nokia lumia 1520',
						'browser second product',
					);

					openCart(context);

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
					assertSuccessfulStep(
						'browser wait for cart delete',
						waitDeleteResult,
					);

					const getCartTextAfterDeleteResult = runBrowserCommand(
						['get', 'text', '#tbodyid', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser get cart text after delete',
						getCartTextAfterDeleteResult,
					);
					expect(getCartTextAfterDeleteResult.output).toMatch(
						/Samsung galaxy s6|Nokia lumia 1520/,
					);

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
				},
			);
		},
		90_000,
	);

	cloudTest(
		'completes checkout flow from cart modal to purchase confirmation',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-scenario-ecommerce-checkout-',
					sessionNamePrefix: 'steel-browser-scenario-ecommerce-checkout',
					commandAttempts: 3,
				},
				({sessionName, environment, projectRoot, startSession, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const context = {
						sessionName,
						environment,
						projectRoot,
						runBrowserCommand,
					};

					const startResult = startSession('browser start');
					assertSuccessfulStep('browser start', startResult);

					addProductToCart(
						context,
						'https://www.demoblaze.com/prod.html?idp_=1',
						'Samsung galaxy s6',
						'browser checkout product',
					);
					openCart(context);

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
					assertSuccessfulStep(
						'browser click place order',
						clickCheckoutResult,
					);

					const waitCheckoutInfoResult = runBrowserCommand(
						['wait', '#name', '--session', sessionName],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser wait for checkout information modal',
						waitCheckoutInfoResult,
					);

					assertSuccessfulStep(
						'browser fill order name',
						runBrowserCommand(
							['fill', '#name', 'Steel', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser fill order country',
						runBrowserCommand(
							['fill', '#country', 'US', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser fill order city',
						runBrowserCommand(
							['fill', '#city', 'NYC', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser fill order card',
						runBrowserCommand(
							['fill', '#card', '4242424242424242', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser fill order month',
						runBrowserCommand(
							['fill', '#month', '12', '--session', sessionName],
							environment,
							projectRoot,
						),
					);
					assertSuccessfulStep(
						'browser fill order year',
						runBrowserCommand(
							['fill', '#year', '2030', '--session', sessionName],
							environment,
							projectRoot,
						),
					);

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

					const waitConfirmationResult = runBrowserCommand(
						[
							'wait',
							'--text',
							'Thank you for your purchase!',
							'--session',
							sessionName,
						],
						environment,
						projectRoot,
					);
					assertSuccessfulStep(
						'browser wait for purchase confirmation',
						waitConfirmationResult,
					);
				},
			);
		},
		90_000,
	);
});
