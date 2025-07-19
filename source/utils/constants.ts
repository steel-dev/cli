import path from 'path';
import os from 'os';
import type {TemplateOptions} from './types.js';

// export const SIGN_IN_URL = 'https://app.steel.dev/sign-in';
// export const SUCCESS_URL = 'https://app.steel.dev/sign-in/cli-success';
// export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys';
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
export const SUCCESS_URL = 'https://cowboys.steel.dev/sign-in/cli-success';
export const TARGET_API_PATH = 'https://steel-api-staging.fly.dev/v1/api-keys';
export const API_PATH = 'https://steel-api-staging.fly.dev/v1';
export const LOGIN_TIMEOUT = 1 * 60 * 1000; // 5 mins
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
