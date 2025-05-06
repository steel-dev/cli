import path from 'path';
import os from 'os';
import {Template} from './types.js';

export const TARGET_SITE = 'https://app.steel.dev/sign-in';
export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys/';
export const API_PATH = 'https://api.steel.dev/v1';
export const CONFIG_DIR = path.join(os.homedir(), '.steel-cli');
export const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');
export const TEMPLATES: Template[] = [
	{
		value: 'steel-playwright-starter-js',
		label: 'Playwright',
	},
	{
		value: 'steel-playwright-starter',
		label: 'Playwright + TypeScript',
	},
	{
		value: 'steel-puppeteer-starter-js',
		label: 'Puppeteer',
	},
	{
		value: 'steel-puppeteer-starter',
		label: 'Puppeteer + TypeScript',
	},
	{
		value: 'steel-files-api-starter',
		label: 'Playwright + Files API Starter in TypeScript',
	},
	{
		value: 'steel-oai-computer-use-node-starter',
		label: 'Steel + OpenAI Computer Use + TypeScript',
	},
	{
		value: 'steel-browser-use-starter',
		label: '(Python) Steel + Browser Use',

		customCommands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install .',
			'python main.py',
		],
		extraEnvVarsRequired: [{value: 'OPENAI_API_KEY', label: 'OpenAI API key'}],
	},
	{
		value: 'steel-oai-computer-use-python-starter',
		label: '(Python) Steel + OpenAI Computer Use',

		customCommands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install .',
			'python main.py',
		],
		extraEnvVarsRequired: [{value: 'OPENAI_API_KEY', label: 'OpenAI API key'}],
	},
	{
		value: 'steel-playwright-python-starter',
		label: '(Python) Steel + Playwright',

		customCommands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install -r requirements.txt',
			'python main.py',
		],
	},
	{
		value: 'steel-selenium-starter',
		label: '(Python) Steel + Selenium',

		customCommands: [
			'python -m venv .venv',
			'source .venv/bin/activate',
			'pip install -r requirements.txt',
			'python main.py',
		],
	},
];

// const TEMPLATE_valueS = TEMPLATES.map(t => t.value);
