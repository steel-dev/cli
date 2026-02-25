import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';

export type BrowserAuthSource = 'env' | 'config' | 'none';

export type BrowserAuthResolution = {
	apiKey: string | null;
	source: BrowserAuthSource;
};

function getConfigPath(environment: NodeJS.ProcessEnv): string {
	const configDirectory =
		environment.STEEL_CONFIG_DIR?.trim() ||
		path.join(os.homedir(), '.config', 'steel');
	return path.join(configDirectory, 'config.json');
}

function readApiKeyFromConfig(environment: NodeJS.ProcessEnv): string | null {
	try {
		const configPath = getConfigPath(environment);
		const configContents = fs.readFileSync(configPath, 'utf-8');
		const parsedConfig = JSON.parse(configContents) as {apiKey?: unknown};

		if (typeof parsedConfig.apiKey === 'string' && parsedConfig.apiKey.trim()) {
			return parsedConfig.apiKey.trim();
		}

		return null;
	} catch {
		return null;
	}
}

export function resolveBrowserAuth(
	environment: NodeJS.ProcessEnv = process.env,
	configApiKey?: string | null,
): BrowserAuthResolution {
	const resolvedConfigApiKey =
		configApiKey === undefined
			? readApiKeyFromConfig(environment)
			: configApiKey;
	const environmentApiKey = environment.STEEL_API_KEY?.trim();
	if (environmentApiKey) {
		return {apiKey: environmentApiKey, source: 'env'};
	}

	const savedApiKey = resolvedConfigApiKey?.trim();
	if (savedApiKey) {
		return {apiKey: savedApiKey, source: 'config'};
	}

	return {apiKey: null, source: 'none'};
}
