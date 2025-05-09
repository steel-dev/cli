import fs from 'node:fs';
import {CONFIG_PATH} from './constants.js';

export function getApiKey(): {
	apiKey: string;
	name: string;
} | null {
	try {
		const config = fs.readFileSync(CONFIG_PATH, 'utf-8');
		const parsedConfig = JSON.parse(config);
		return parsedConfig;
	} catch (error) {
		return null;
	}
}
