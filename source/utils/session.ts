import fs from 'node:fs';
import {CONFIG_PATH} from './constants.js';

export function getApiKey(): {
	apiKey: string;
	name: string;
} | null {
	try {
		const config = fs.readFileSync(CONFIG_PATH, 'utf-8');
		const parsedConfig = JSON.parse(config);
		if (parsedConfig && parsedConfig.apiKey && parsedConfig.name) {
			return parsedConfig;
		}
		return null;
	} catch (error) {
		return null;
	}
}
