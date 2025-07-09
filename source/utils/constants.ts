import path from 'path';
import os from 'os';
import {Template} from './types.js';

export const TARGET_SITE = 'https://app.steel.dev/sign-in';
export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys/';
export const API_PATH = 'https://api.steel.dev/v1';
export const LOCAL_API_PATH = 'http://localhost:3000/v1';
export const CONFIG_DIR = path.join(os.homedir(), '.steel-cli');
export const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');
export const REPO_URL = 'https://github.com/steel-dev/steel-browser.git';
export const TEMPLATES: Template[] = [
	{
		alias: 'playwright-js',
		label: 'Playwright',
		value: 'steel-playwright-starter-js',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'playwright',
		label: 'Playwright + TypeScript',
		value: 'steel-playwright-starter',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'puppeteer-js',
		label: 'Puppeteer',
		value: 'steel-puppeteer-starter-js',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'puppeteer',
		value: 'steel-puppeteer-starter',
		label: 'Puppeteer + TypeScript',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'puppeteer',
		value: 'steel-puppeteer-starter',
		label: 'Puppeteer + TypeScript',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'files',
		value: 'steel-files-api-starter',
		label: 'Playwright + Files API Starter in TypeScript',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'oai-cua',
		value: 'steel-oai-computer-use-starter',
		label: 'Steel + OpenAI Computer Use + TypeScript',
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'OPENAI_API_KEY', label: 'OpenAI API key', required: true},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
			{value: 'TASK', label: 'Task for the agent'},
		],
		commands: ['npm install', 'npm run dev'],
	},
	{
		alias: 'browser-use',
		value: 'steel-browser-use-starter',
		label: '(Python) Steel + Browser Use',

		commands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install .',
			'python main.py',
		],
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

		commands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install .',
			'python main.py',
		],
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

		commands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install -r requirements.txt',
			'python main.py',
		],
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

		commands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install -r requirements.txt',
			'python main.py',
		],
		env: [
			{value: 'STEEL_API_KEY', label: 'Steel API key'},
			{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
			{value: 'STEEL_API_URL', label: 'Steel API URL'},
		],
	},
];

// const TEMPLATE_values = TEMPLATES.map(t => t.value);
