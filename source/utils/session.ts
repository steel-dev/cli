import fs from 'fs/promises';
import {CONFIG_PATH} from './constants.js';

export async function getApiKey(): Promise<{
	apiKey: string;
	name: string;
} | null> {
	try {
		const config = await fs.readFile(CONFIG_PATH, 'utf-8');
		const parsedConfig = JSON.parse(config);
		return parsedConfig;
	} catch (error) {
		return null;
	}
}
