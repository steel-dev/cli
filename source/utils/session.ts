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
			return {apiKey: parsedConfig.apiKey, name: parsedConfig.name};
		}
		return null;
	} catch (error) {
		return null;
	}
}

export function getSettings(): {
	instance: string;
} | null {
	try {
		const config = fs.readFileSync(CONFIG_PATH, 'utf-8');
		const parsedConfig = JSON.parse(config);
		if (parsedConfig && parsedConfig.instance) {
			return {instance: parsedConfig.instance};
		} else {
			setSettings({instance: 'local'});
			return {instance: 'local'};
		}
	} catch (error) {
		return null;
	}
}

export function setSettings(value: any): void {
	try {
		const config = fs.readFileSync(CONFIG_PATH, 'utf-8');
		const parsedConfig = JSON.parse(config);
		for (const key in value) {
			parsedConfig[key] = value[key];
		}
		fs.writeFileSync(CONFIG_PATH, JSON.stringify(parsedConfig));
	} catch (error) {
		console.error('Error setting settings:', error);
	}
}
