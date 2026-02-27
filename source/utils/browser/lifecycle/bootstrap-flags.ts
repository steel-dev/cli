import {BrowserAdapterError} from '../errors.js';
import {
	parsePositiveIntegerFlagValue,
	resolveExplicitApiUrl,
} from './api-client.js';
import type {ParsedBootstrapOptions} from './types.js';

export function parseBrowserPassthroughBootstrapFlags(browserArgv: string[]): {
	options: ParsedBootstrapOptions;
	passthroughArgv: string[];
} {
	const options: ParsedBootstrapOptions = {
		local: false,
		apiUrl: null,
		sessionName: null,
		stealth: false,
		proxyUrl: null,
		timeoutMs: null,
		headless: false,
		region: null,
		solveCaptcha: false,
		autoConnect: false,
		cdpTarget: null,
	};
	const passthroughArgv: string[] = [];

	for (let index = 0; index < browserArgv.length; index++) {
		const argument = browserArgv[index];

		if (argument === '--local') {
			options.local = true;
			continue;
		}

		if (argument === '--stealth') {
			options.stealth = true;
			continue;
		}

		if (argument === '--api-url' || argument.startsWith('--api-url=')) {
			const value =
				argument === '--api-url'
					? browserArgv[index + 1]
					: argument.slice('--api-url='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --api-url.',
				);
			}

			options.apiUrl = resolveExplicitApiUrl(value);

			if (argument === '--api-url') {
				index++;
			}

			continue;
		}

		if (argument === '--session' || argument.startsWith('--session=')) {
			const value =
				argument === '--session'
					? browserArgv[index + 1]
					: argument.slice('--session='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session.',
				);
			}

			options.sessionName = value.trim();

			if (argument === '--session') {
				index++;
			}

			continue;
		}

		if (argument === '--proxy' || argument.startsWith('--proxy=')) {
			const value =
				argument === '--proxy'
					? browserArgv[index + 1]
					: argument.slice('--proxy='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --proxy.',
				);
			}

			options.proxyUrl = value.trim();

			if (argument === '--proxy') {
				index++;
			}

			continue;
		}

		if (
			argument === '--session-timeout' ||
			argument.startsWith('--session-timeout=')
		) {
			const value =
				argument === '--session-timeout'
					? browserArgv[index + 1]
					: argument.slice('--session-timeout='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-timeout.',
				);
			}

			options.timeoutMs = parsePositiveIntegerFlagValue(
				value,
				'--session-timeout',
			);

			if (argument === '--session-timeout') {
				index++;
			}

			continue;
		}

		if (argument === '--session-headless') {
			options.headless = true;
			continue;
		}

		if (argument.startsWith('--session-headless=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--session-headless` does not accept a value.',
			);
		}

		if (
			argument === '--session-region' ||
			argument.startsWith('--session-region=')
		) {
			const value =
				argument === '--session-region'
					? browserArgv[index + 1]
					: argument.slice('--session-region='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-region.',
				);
			}

			const normalizedRegion = value.trim();
			if (!normalizedRegion) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --session-region.',
				);
			}

			options.region = normalizedRegion;

			if (argument === '--session-region') {
				index++;
			}

			continue;
		}

		if (argument === '--session-solve-captcha') {
			options.solveCaptcha = true;
			continue;
		}

		if (argument.startsWith('--session-solve-captcha=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--session-solve-captcha` does not accept a value.',
			);
		}

		if (argument === '--auto-connect') {
			options.autoConnect = true;
			passthroughArgv.push('--auto-connect');
			continue;
		}

		if (argument.startsWith('--auto-connect=')) {
			throw new BrowserAdapterError(
				'INVALID_BROWSER_ARGS',
				'`--auto-connect` does not accept a value. Use `--cdp <url|port>` for explicit endpoints.',
			);
		}

		if (argument === '--cdp' || argument.startsWith('--cdp=')) {
			const value =
				argument === '--cdp'
					? browserArgv[index + 1]
					: argument.slice('--cdp='.length);

			if (!value) {
				throw new BrowserAdapterError(
					'INVALID_BROWSER_ARGS',
					'Missing value for --cdp.',
				);
			}

			options.cdpTarget = value.trim();
			passthroughArgv.push('--cdp', value.trim());

			if (argument === '--cdp') {
				index++;
			}

			continue;
		}

		passthroughArgv.push(argument);
	}

	if (options.autoConnect && options.cdpTarget) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'Cannot combine `--auto-connect` with `--cdp`.',
		);
	}

	return {
		options,
		passthroughArgv,
	};
}
