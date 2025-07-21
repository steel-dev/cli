import fs from 'node:fs';
import {CONFIG_PATH, CONFIG_DIR} from './constants.js';

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
	} catch {
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
		console.error(error);
		console.log('No settings found, setting to local');
		setSettings({instance: 'local'});
		return {instance: 'local'};
	}
}

export function setSettings(value: object): void {
	try {
		// Ensure config directory exists
		fs.mkdirSync(CONFIG_DIR, {recursive: true});

		let parsedConfig = {};
		try {
			const config = fs.readFileSync(CONFIG_PATH, 'utf-8');
			parsedConfig = JSON.parse(config);
		} catch {
			// Config file doesn't exist, start with empty object
		}

		for (const key in value) {
			parsedConfig[key] = value[key];
		}
		fs.writeFileSync(CONFIG_PATH, JSON.stringify(parsedConfig));
	} catch (error) {
		console.error('Error setting settings:', error);
	}
}
