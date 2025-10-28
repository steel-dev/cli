import path from 'path';
import os from 'os';
import type {TemplateOptions, Template} from './types.js';
import {loadManifest, convertManifestToTemplates} from './registry.js';

export const LOGIN_URL = 'https://app.steel.dev/sign-in';
export const SIGN_IN_URL = 'https://app.steel.dev/sign-in';
export const SUCCESS_URL = 'https://app.steel.dev/sign-in/cli-success';
export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys';
export const API_PATH = 'https://api.steel.dev/v1';
export const LOCAL_API_PATH = 'http://localhost:3000/v1';
export const CONFIG_DIR = path.join(os.homedir(), '.config', 'steel');
export const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');
export const CACHE_DIR = path.join(os.homedir(), '.cache', 'steel');
export const REPO_URL = 'https://github.com/steel-dev/steel-browser.git';
export const LOGIN_TIMEOUT = 1 * 60 * 1000; // 5 mins
export const ENV_VAR_MAP = {
	api_key: 'STEEL_API_KEY',
	openai_key: 'OPENAI_API_KEY',
	anthropic_key: 'ANTHROPIC_API_KEY',
	api_url: 'STEEL_API_URL',
	task: 'TASK',
};

let _templates: Template[] | null = null;

export function getTemplates(): Template[] {
	if (!_templates) {
		try {
			const manifest = loadManifest();
			_templates = convertManifestToTemplates(manifest);
		} catch (error) {
			console.error('Failed to load templates from manifest:', error);
			_templates = [];
		}
	}
	return _templates;
}

export const TEMPLATES = getTemplates();
// const TEMPLATE_values = TEMPLATES.map(t => t.value);
