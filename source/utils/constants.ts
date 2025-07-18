import path from 'path';
import os from 'os';
import type {TemplateOptions} from './types.js';

export const TARGET_SITE = 'https://app.steel.dev/sign-in';
// export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys/';
// export const API_PATH = 'https://api.steel.dev/v1';
export const LOCAL_API_PATH = 'http://localhost:3000/v1';
export const CONFIG_DIR = path.join(os.homedir(), '.config', 'steel');
export const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');
export const CACHE_DIR = path.join(os.homedir(), '.cache', 'steel');
// export const NODE_MODULES_DIR = path.join(CACHE_DIR, 'node_modules');
// export const PYTHON_VENV_DIR = path.join(CACHE_DIR, 'venvs');
export const REPO_URL = 'https://github.com/steel-dev/steel-browser.git';
// export const LOGIN_URL = 'https://app.steel.dev/sign-in';
export const LOGIN_URL = 'https://cowboys.steel.dev/sign-in';
// export const SUCCESS_URL = 'https://app.steel.dev/sign-in/cli-success';
// export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys';
// export const API_PATH = 'https://api.steel.dev/v1';
// export const LOCAL_API_PATH = 'http://localhost:3000/v1';

export const SUCCESS_URL = 'https://cowboys.steel.dev/sign-in/cli-success';
export const TARGET_API_PATH = 'https://steel-api-staging.fly.dev/v1/api-keys';
export const API_PATH = 'https://steel-api-staging.fly.dev/v1';
export const LOGIN_TIMEOUT = 5 * 60 * 1000; // 5 mins
export const ENV_VAR_MAP = {
	api_key: 'STEEL_API_KEY',
	openai_key: 'OPENAI_API_KEY',
	api_url: 'STEEL_API_URL',
	task: 'TASK',
};

export const TEMPLATES = [
	{
		alias: 'playwright-js',
		label: 'Playwright',
		value: 'steel-playwright-starter-js',
		language: 'JS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) =>
			`NODE_PATH=${options.depsDir}/node_modules node index.js`,
	},
	{
		alias: 'playwright',
		label: 'Playwright + TypeScript',
		value: 'steel-playwright-starter',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'puppeteer-js',
		label: 'Puppeteer',
		value: 'steel-puppeteer-starter-js',
		language: 'JS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) =>
			`NODE_PATH=${options.depsDir}/node_modules node index.js`,
	},
	{
		alias: 'puppeteer',
		value: 'steel-puppeteer-starter',
		label: 'Puppeteer + TypeScript',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'files',
		value: 'steel-files-api-starter',
		label: 'Playwright + Files API Starter in TypeScript',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'creds',
		value: 'steel-credentials-starter',
		label: 'Playwright + Credentials API Starter in TypeScript',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'oai-cua',
		value: 'steel-oai-computer-use-starter',
		label: 'Steel + OpenAI Computer Use + TypeScript',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'OPENAI_API_KEY', label: 'OpenAI API key', required: true},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
			{value: 'TASK', label: 'Task for the agent'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'magnitude',
		value: 'steel-magnitude-starter',
		label: 'Steel + Magnitude',
		language: 'TS',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'ANTHROPIC_API_KEY', label: 'Anthropic API key', required: true},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
			{value: 'TASK', label: 'Task for the agent'},
		],
		depCommands: (options: TemplateOptions) => [
			`mkdir -p ${options.depsDir}`,
			`cp package.json ${options.depsDir}/package.json`,
			`cd ${options.depsDir} && npm install --prefer-offline --no-audit --silent`,
		],
		runCommand: (options: TemplateOptions) => {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});

			return `${nodePath} ts-node --compiler-options '${compilerOptions}' index.ts`;
		},
	},
	{
		alias: 'browser-use',
		value: 'steel-browser-use-starter',
		label: '(Python) Steel + Browser Use',
		language: 'PY',
		depCommands: (options: TemplateOptions) => [
			`python3 -m venv ${options.depsDir}`,
			`${options.depsDir}/bin/pip install -r requirements.txt`,
		],
		runCommand: (options: TemplateOptions) =>
			`${options.depsDir}/bin/python3 main.py`,
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'OPENAI_API_KEY', label: 'OpenAI API key', required: true},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
			{value: 'TASK', label: 'Task for the agent'},
		],
	},
	{
		alias: 'oai-cua-py',
		value: 'steel-oai-computer-use-python-starter',
		label: '(Python) Steel + OpenAI Computer Use',
		language: 'PY',
		depCommands: (options: TemplateOptions) => [
			`python3 -m venv ${options.depsDir}`,
			`${options.depsDir}/bin/pip install -r requirements.txt`,
		],
		runCommand: (options: TemplateOptions) =>
			`${options.depsDir}/bin/python3 main.py`,
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'OPENAI_API_KEY', label: 'OpenAI API key', required: true},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
			{value: 'TASK', label: 'Task for the agent'},
		],
	},
	{
		alias: 'playwright-py',
		value: 'steel-playwright-python-starter',
		label: '(Python) Steel + Playwright',
		language: 'PY',
		depCommands: (options: TemplateOptions) => [
			`python3 -m venv ${options.depsDir}`,
			`${options.depsDir}/bin/pip install -r requirements.txt`,
		],
		runCommand: (options: TemplateOptions) =>
			`${options.depsDir}/bin/python3 main.py`,
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
	},
	{
		alias: 'selenium',
		value: 'steel-selenium-starter',
		label: '(Python) Steel + Selenium',
		language: 'PY',
		depCommands: (options: TemplateOptions) => [
			`python3 -m venv ${options.depsDir}`,
			`${options.depsDir}/bin/pip install -r requirements.txt`,
		],
		runCommand: (options: TemplateOptions) =>
			`${options.depsDir}/bin/python3 main.py`,
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
	},
];
// const TEMPLATE_values = TEMPLATES.map(t => t.value);

export const SUCCESS_HTML = `<!doctype html>
<html lang="en">
	<head>
		<meta charset="UTF-8" />
		<title>Authentication Successful</title>
		<link
			href="https://fonts.googleapis.com/css2?family=JetBrains+Mono&display=swap"
			rel="stylesheet"
		/>
		<style>
			body {
				margin: 0;
				padding: 0;
				height: 100vh;
				background-color: #000;
				color: #e0e0e0;
				display: flex;
				justify-content: center;
				align-items: center;
				font-family: 'JetBrains Mono', monospace;
			}

			.container {
				text-align: center;
				background: #000;
				padding: 50px 40px;
				border-color: rgb(118, 109, 95);
				border-width: 2px;
				border-style: solid;
				max-width: 400px;
				width: 90%;
			}

			.logo {
				max-width: 180px;
				margin-bottom: 30px;
			}

			h1 {
				color: #fbe32d;
				font-size: 2em;
				margin-bottom: 15px;
			}

			p {
				font-size: 1em;
				color: #eeeeec;
				margin-bottom: 30px;
			}

			.terminal-hint {
				display: inline-block;
				padding: 10px 20px;
				background-color: #1f2937;
				border: 1px solid #374151;
				border-radius: 6px;
				color: #9ca3af;
				font-size: 0.9em;
			}
			.social-links {
				margin-top: 20px;
			}

			.social-links a {
				margin: 0 10px;
				display: inline-block;
				transition: transform 0.2s ease;
			}

			.social-links a:hover {
				transform: scale(1.05);
			}

			.social-links img {
				width: 28px;
				height: 28px;
				filter: brightness(0) invert(1);
				opacity: 0.7;
			}
		</style>
	</head>
	<body>
		<div class="container">
			<img
				src="https://github.com/steel-dev/steel-browser/raw/main/images/steel_header_logo.png"
				alt="Steel Logo"
				class="logo"
			/>
			<h1>Authentication Successful</h1>
			<p>You can now return to your terminal.</p>
			<div class="social-links">
				<a
					href="https://github.com/steel-dev/steel-browser"
					target="_blank"
					title="GitHub"
				>
					<img
						src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg"
						alt="GitHub"
					/>
				</a>
				<a
					href="https://discord.gg/YOUR_INVITE_CODE"
					target="_blank"
					title="Discord"
				>
					<img
						src="https://cdn-icons-png.flaticon.com/512/5968/5968756.png"
						alt="Discord"
					/>
				</a>
			</div>
		</div>
	</body>
</html>
`;
