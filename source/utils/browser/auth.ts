import {readApiKeyFromConfig} from './apiConfig.js';

export type BrowserAuthSource = 'env' | 'config' | 'none';

export type BrowserAuthResolution = {
	apiKey: string | null;
	source: BrowserAuthSource;
};

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
