//@ts-nocheck
import fs from 'fs';
import {
	appendToTopofFile,
	replaceString,
	wrapStringinFile,
	fileParse,
	indentation,
	insertCode,
} from './files.js';

import {parse, print, types} from 'recast';
import tsParser from 'recast/parsers/typescript.js';
//@ts-ignore
import {namedTypes as n, builders as b} from 'ast-types';

const requiredImportsPuppeteerJs = [
	/from ['"]puppeteer-core['"]/,
	/from ['"]puppeteer['"]/,
];
const codePatternsPuppeteerJs = [/puppeteer.launch()/];

const configPuppeteerJs = (file: string) => {
	const content = fs.readFileSync(file, 'utf8');

	// Helper: create `await client.sessions.create({})`
	function createSessionExpr() {
		return b.awaitExpression(
			b.callExpression(
				b.memberExpression(
					b.memberExpression(b.identifier('client'), b.identifier('sessions')),
					b.identifier('create'),
				),
				[b.objectExpression([])],
			),
		);
	}

	// Helper: insert let session; at top of function
	//@ts-ignore
	function insertSessionDeclaration(
		path: types.NodePath<types.namedTypes.Declaration, any>,
	) {
		if (sessionDeclared) return;

		const body = path?.node?.body?.body;
		const sessionDecl = b.variableDeclaration('let', [
			b.variableDeclarator(b.identifier('session'), null),
		]);
		body.unshift(sessionDecl);
		sessionDeclared = true;
	}

	const ast = parse(content, {
		parser: tsParser,
	});

	let puppeteerImportName: any;
	let browserName: string | null = null;
	let sessionDeclared = false;

	// Traverse to extract puppeteer import name and add new import
	types.visit(ast, {
		visitImportDeclaration(path) {
			const node = path.node;

			// Find Puppeteer import
			if (
				(node?.source?.value === 'puppeteer' ||
					node?.source?.value === 'puppeteer-core') &&
				node?.specifiers?.length === 1 &&
				node?.specifiers[0]?.type === 'ImportDefaultSpecifier'
			) {
				puppeteerImportName = node?.specifiers[0]?.local?.name;
			}

			this.traverse(path);
		},
		visitProgram(path) {
			const body = path.node.body;

			// Add: import { load_dotenv } from 'dotenv';
			const dotenvImport = b.importDeclaration(
				[b.importSpecifier(b.identifier('load_dotenv'))],
				b.literal('dotenv'),
			);

			// add import Steel from 'steel-dev';
			const steelDevImport = b.importDeclaration(
				[b.importSpecifier(b.identifier('Steel'))],
				b.literal('steel-dev'),
			);

			// Insert after the last import
			const lastImportIndex = body.findIndex(
				node => node.type !== 'ImportDeclaration',
			);
			body.splice(lastImportIndex, 0, dotenvImport as any);
			body.splice(lastImportIndex + 1, 0, steelDevImport as any);

			// Step 1: Detect existing session declaration
			types.visit(ast, {
				visitVariableDeclarator(path) {
					if (
						n.Identifier.check(path.node.id) &&
						path.node.id.name === 'session'
					) {
						sessionDeclared = true;
					}
					this.traverse(path);
				},
			});

			// Step 2: Detect `let browser;`
			types.visit(ast, {
				visitVariableDeclaration(path) {
					const decl: any = path.node.declarations[0];
					if (
						n.Identifier.check(decl?.id) &&
						decl.init === null &&
						path.node.kind === 'let'
					) {
						browserName = decl.id.name;
					}
					this.traverse(path);
				},
			});

			// Step 3: Handle both inline and separate assignment
			types.visit(ast, {
				visitVariableDeclaration(path) {
					const decl: any = path.node.declarations[0];

					if (
						n.Identifier.check(decl.id) &&
						n.AwaitExpression.check(decl.init) &&
						n.CallExpression.check(decl.init.argument)
					) {
						const call = decl.init.argument;
						if (
							n.MemberExpression.check(call.callee) &&
							call.callee.object.name === puppeteerImportName &&
							['launch', 'connect'].includes(call.callee.property.name)
						) {
							browserName = decl.id.name;
							const parent = path.parentPath;

							if (n.BlockStatement.check(parent.node)) {
								const i = parent.node.body.indexOf(path.node);
								parent.node.body.splice(
									i + 1,
									0,
									b.variableDeclaration('const', [
										b.variableDeclarator(
											b.identifier('session'),
											createSessionExpr(),
										),
									]),
								);
							}
						}
					}
					this.traverse(path);
				},

				visitAssignmentExpression(path) {
					if (
						browserName &&
						n.Identifier.check(path.node.left) &&
						path.node.left.name === browserName &&
						n.AwaitExpression.check(path.node.right) &&
						n.CallExpression.check(path.node.right.argument)
					) {
						const call = path.node.right.argument;
						if (
							n.MemberExpression.check(call.callee) &&
							call?.callee?.object?.name === puppeteerImportName &&
							['launch', 'connect'].includes(call.callee.property.name)
						) {
							// Add `let session;` at top of function if needed
							const fn = path.getFunctionParent();
							if (fn && n.FunctionDeclaration.check(fn.node)) {
								insertSessionDeclaration(fn);
							}

							// Insert `session = await client.sessions.create({})`
							const parent = path.parentPath.parentPath;
							if (n.BlockStatement.check(parent.node)) {
								const i = parent.node.body.indexOf(path.parentPath.node);
								parent.node.body.splice(
									i + 1,
									0,
									b.expressionStatement(
										b.assignmentExpression(
											'=',
											b.identifier('session'),
											createSessionExpr(),
										),
									),
								);
							}
						}
					}

					this.traverse(path);
				},
			});
		},
	});
	// 	appendToTopofFile(
	// 		file,
	// 		`import Steel from "steel-sdk";
	// import dotenv from "dotenv";

	// dotenv.config();

	// const STEEL_API_KEY = process.env.STEEL_API_KEY;
	// // Initialize Steel client with the API key from environment variables
	// const client = new Steel({
	//   steelAPIKey: STEEL_API_KEY,
	// });`,
	// 	);
	// 	replaceString(
	// 		file,
	// 		'await puppeteer.launch()',
	// 		` console.log("Creating Steel session...");

	//     // Create a new Steel session with all available options
	//     session = await client.sessions.create({
	//       // === Basic Options ===
	//       // useProxy: true, // Use Steel's proxy network (residential IPs)
	//       // proxyUrl: 'http://...',         // Use your own proxy (format: protocol://username:password@host:port)
	//       // solveCaptcha: true,             // Enable automatic CAPTCHA solving
	//       // sessionTimeout: 1800000,        // Session timeout in ms (default: 5 mins)
	//       // === Browser Configuration ===
	//       // userAgent: 'custom-ua-string',  // Set a custom User-Agent
	//     });

	//     console.log(
	//       \`\x1b[1;93mSteel Session created!\x1b[0m\n` +
	// 			`View session at \x1b[1;37m\${session.sessionViewerUrl}\x1b[0m\`
	//     );

	//     // Connect Puppeteer to the Steel session
	//     browser = await puppeteer.connect({
	//       browserWSEndpoint: \`wss://connect.steel.dev?apiKey=\${STEEL_API_KEY}&sessionId=\${session.id}\`,
	//     });

	//     console.log("Connected to browser via Puppeteer");`,
	// 	);
	// 	wrapStringinFile(
	// 		file,
	// 		'',
	// 		'await browser.close()',
	// 		` if (session) {
	//       console.log("Releasing session...");
	//       await client.sessions.release(session.id);
	//       console.log("Session released");
	//     }`,
	// 	);
	// Print the modified code
	console.log('\nModified code:\n');
	console.log(print(ast).code);
	console.log('\nDetected Puppeteer import name:', puppeteerImportName);
};

const requiredImportsPlaywrightJs = [/from ['"]playwright['"]/];
const codePatternsPlaywrightJs = [/launch()/];
const configPlaywrightJs = (file: string) => {
	appendToTopofFile(
		file,
		`import Steel from "steel-sdk";
import dotenv from "dotenv";

dotenv.config();

const STEEL_API_KEY = process.env.STEEL_API_KEY;
// Initialize Steel client with the API key from environment variables
const client = new Steel({
  steelAPIKey: STEEL_API_KEY,
});`,
	);
	replaceString(
		file,
		'await puppeteer.launch()',
		` console.log("Creating Steel session...");

    // Create a new Steel session with all available options
    session = await client.sessions.create({
      // === Basic Options ===
      // useProxy: true, // Use Steel's proxy network (residential IPs)
      // proxyUrl: 'http://...',         // Use your own proxy (format: protocol://username:password@host:port)
      // solveCaptcha: true,             // Enable automatic CAPTCHA solving
      // sessionTimeout: 1800000,        // Session timeout in ms (default: 5 mins)
      // === Browser Configuration ===
      // userAgent: 'custom-ua-string',  // Set a custom User-Agent
    });

    console.log(
      \`\x1b[1;93mSteel Session created!\x1b[0m\n` +
			`View session at \x1b[1;37m\${session.sessionViewerUrl}\x1b[0m\`
    );

    // Connect Puppeteer to the Steel session
    browser = await puppeteer.connect({
      browserWSEndpoint: \`wss://connect.steel.dev?apiKey=\${STEEL_API_KEY}&sessionId=\${session.id}\`,
    });

    console.log("Connected to browser via Puppeteer");`,
	);
	wrapStringinFile(
		file,
		'',
		'await browser.close()',
		` if (session) {
      console.log("Releasing session...");
      await client.sessions.release(session.id);
      console.log("Session released");
    }`,
	);
};

const requiredImportsPlaywrightPy = [/from playwright/];
const codePatternsPlaywrightPy = [/launch()/, /playwright/];
const configPlaywrightPy = (file: string) => {
	//@ts-ignore
	const {imports, body, exports} = fileParse(file);

	const addedImports = [
		'from steel import Steel',
		'from dotenv import load_dotenv',
	].filter(item => !imports.includes(item));

	imports.unshift(...addedImports);

	// Code to add to top of body, remove duplicates
	const addedTopOfBody = [
		'load_dotenv()',
		"STEEL_API_KEY = os.getenv('STEEL_API_KEY')",
		'# Initialize Steel client with the API key from environment variables',
		'client = Steel(steel_api_key=STEEL_API_KEY)',
		'print("Creating Steel session...")',
		'# Create a new Steel session with all available options',
		`session = client.sessions.create(
		# === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
    )`,
	].filter(item => !body.includes(item));

	body.unshift(...addedTopOfBody);

	insertCode(
		'playwright = ',
		[
			'# Connect Playwright to the Steel session',
			'browser = playwright.chromium.connect_over_cdp(f"wss://connect.steel.dev?apiKey={STEEL_API_KEY}&sessionId={session.id}")',
			'print("Connected to browser via Playwright")',
		],
		body,
	);

	insertCode(
		'browser.close()',
		[
			'if session:',
			'    print("Releasing session...")',
			'    client.sessions.release(session.id)',
			'    print("Session released")',
		],
		body,
	);

	// Write the changes to the file
	fs.writeFile(
		file,
		imports.join('\n') + '\n\n' + body.join('\n') + '\n\n' + exports.join('\n'),
		'utf8',
		err => {
			if (err) {
				console.error('Error writing file:', err);
				return;
			}
			console.log(`${file}: File updated successfully!`);
		},
	);
	// console.log(body);
};

const requiredImportsBrowserUsePy = [/from browser_use import Agent/];
const codePatternsBrowserUsePy = [/Agent/];
const configBrowserUsePy = (file: string) => {
	const {imports, body, exports} = fileParse(file);
	const addedImports = [
		'import os',
		'from dotenv import load_dotenv',
		'from steel import Steel',
	].filter(item => !imports.includes(item));

	imports.unshift(...addedImports);

	const browserUseImport: string = 'from browser_use import Agent';
	// replace import from browser_user with one that uses the new things that we need, Agent, Browser, BrowserContext
	let foundIndex: number = imports.findIndex(item =>
		item.includes(browserUseImport),
	);

	if (foundIndex !== -1) {
		imports[foundIndex] =
			'from browser_use import Agent, Browser, BrowserContext, BrowserConfig';
	}

	const addedTopOfBody = [
		'load_dotenv()',
		"STEEL_API_KEY = os.getenv('STEEL_API_KEY')",
		'# Initialize Steel client with the API key from environment variables',
		'client = Steel(steel_api_key=STEEL_API_KEY)',
		'print("Creating Steel session...")',
		'# Create a new Steel session with all available options',
		`session = client.sessions.create(
		# === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
    )`,
		'cdp_url = f"wss://connect.steel.dev?apiKey={STEEL_API_KEY}&sessionId={session.id}"',
		'browser = Browser(config=BrowserConfig(cdp_url=cdp_url))',
		'browser_context = BrowserContext(browser=browser)',
	].filter(item => !body.includes(item));

	// How can I add config when I am inside the agent configuration, how can I even tell when I'm inside?
	// Okay the Agent config needs the parentheses to find the close, we can search for the Agent keyword and get the index, find where it ends and get that index and if it is in the same line then update
	// if not then insert two new lines.

	const addedAgentConfig = [
		'browser=browser',
		'browser_context=browser_context',
	];

	body.forEach((item, index, arr) => {
		// Searching through the body and looking for the declaration of Agent
		// Can't be in a comment so substrings the text from the comments before searching

		let commentIndex: number = Math.min(item.indexOf('#'));
		if (commentIndex === -1) {
			commentIndex = item.length;
		}
		let substring: string = item.substring(0, commentIndex);
		if (substring.includes('Agent(')) {
			// We found the Agent declaration, now we need to see if it ends on this line or on another line
			let agentIndex: number = substring.indexOf('Agent(');
			let restSubstring = substring.substring(agentIndex);
			let closingAgentIndex: number = restSubstring.indexOf(')');
			if (closingAgentIndex === -1) {
				// Add in the required config if the Agent config doesn't end on one line
				const indent = indentation(item, index, arr);
				body.splice(
					index + 1,
					0,
					' '.repeat(indent) + 'browser_context=browser_context,',
				);
				body.splice(index + 1, 0, ' '.repeat(indent) + 'browser=browser,');
			} else {
				// The Agent config does end on the same line
				// Split the config up by commas and then add them in
				let configSubstring: string = restSubstring.substring(
					0,
					closingAgentIndex,
				);
				const agentConfig = configSubstring.split(',');
				// Add in the added config
				addedAgentConfig.forEach(item => {
					if (!agentConfig.includes(item)) {
						agentConfig.unshift(item);
					}
				});
				configSubstring = agentConfig.join(',');
				arr[index] =
					item.substring(0, agentIndex) +
					configSubstring +
					')' +
					item.substring(closingAgentIndex);
			}
			return;
		}
	});

	body.unshift(...addedTopOfBody);

	// Write the changes to the file
	fs.writeFile(
		file,
		imports.join('\n') + '\n\n' + body.join('\n') + '\n\n' + exports.join('\n'),
		'utf8',
		err => {
			if (err) {
				console.error('Error writing file:', err);
				return;
			}
			console.log(`${file}: File updated successfully!`);
		},
	);
};

const requiredImportsSeleniumPy = [/from selenium/];
const codePatternsSeleniumPy = [/webdriver/];
const configSeleniumPy = (file: string) => {
	appendToTopofFile(
		file,
		`
		from steel import Steel
from dotenv import load_dotenv
from selenium.webdriver.remote.remote_connection import RemoteConnection

load_dotenv()
STEEL_API_KEY = os.getenv('STEEL_API_KEY')

# Initialize Steel client with the API key from environment variables
client = Steel(
    steel_api_key=STEEL_API_KEY,
)

class CustomRemoteConnection(RemoteConnection):
    _session_id = None

    def __init__(self, remote_server_addr: str, session_id: str):
        super().__init__(remote_server_addr)
        self._session_id = session_id

    def get_remote_connection_headers(self, parsed_url, keep_alive=False):
        headers = super().get_remote_connection_headers(parsed_url, keep_alive)
        headers.update({'steel-api-key': os.environ.get("STEEL_API_KEY")})
        headers.update({'session-id': self._session_id})
        return headers


print("Creating Steel session...")

# Create a new Steel session with all available options
session = client.sessions.create(
    # === Basic Options ===
    # use_proxy=True,              # Use Steel's proxy network (residential IPs)
    # proxy_url='http://...',      # Use your own proxy (format: protocol://username:password@host:port)
    # solve_captcha=True,          # Enable automatic CAPTCHA solving
    # session_timeout=1800000,     # Session timeout in ms (default: 5 mins)
    # === Browser Configuration ===
    # user_agent='custom-ua',      # Set a custom User-Agent
    is_selenium=True,
)

# Connect to the session via Selenium's WebDriver using the CustomRemoteConnection class
        driver = webdriver.Remote(
            command_executor=CustomRemoteConnection(
                remote_server_addr='http://connect.steelbrowser.com/selenium',
                session_id=session.id
            ),
            options=webdriver.ChromeOptions()
        )
`,
	);
};

export const possibleJsImports = [
	{
		name: 'puppeteer',
		imports: requiredImportsPuppeteerJs,
		codePatterns: codePatternsPuppeteerJs,
		config: configPuppeteerJs,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightJs,
		codePatterns: codePatternsPlaywrightJs,
		config: configPlaywrightJs,
	},
];
export const possiblePyImports = [
	{
		name: 'browser-use',
		imports: requiredImportsBrowserUsePy,
		codePatterns: codePatternsBrowserUsePy,
		config: configBrowserUsePy,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightPy,
		codePatterns: codePatternsPlaywrightPy,
		config: configPlaywrightPy,
	},
	{
		name: 'selenium',
		imports: requiredImportsSeleniumPy,
		codePatterns: codePatternsSeleniumPy,
		config: configSeleniumPy,
	},
];
