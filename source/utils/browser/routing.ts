export type BrowserDispatchTarget = 'none' | 'native' | 'passthrough';

const STEEL_GLOBAL_FLAGS = new Set(['--no-update-check']);
const NATIVE_BROWSER_SUBCOMMANDS = new Set([
	'start',
	'stop',
	'sessions',
	'live',
]);

export function filterSteelGlobalFlags(argv: string[]): string[] {
	return argv.filter(argument => !STEEL_GLOBAL_FLAGS.has(argument));
}

function getCommandTokens(argv: string[]): string[] {
	return argv.filter(argument => !argument.startsWith('-'));
}

export function isBrowserCommand(argv: string[]): boolean {
	const commandTokens = getCommandTokens(filterSteelGlobalFlags(argv));
	return commandTokens[0] === 'browser';
}

export function resolveBrowserDispatchTarget(
	argv: string[],
): BrowserDispatchTarget {
	const commandTokens = getCommandTokens(filterSteelGlobalFlags(argv));

	if (commandTokens[0] !== 'browser') {
		return 'none';
	}

	const subcommand = commandTokens[1];
	if (!subcommand || NATIVE_BROWSER_SUBCOMMANDS.has(subcommand)) {
		return 'native';
	}

	return 'passthrough';
}

export function getBrowserPassthroughArgv(argv: string[]): string[] {
	const filteredArgv = filterSteelGlobalFlags(argv);
	const browserIndex = filteredArgv.findIndex(
		argument => argument === 'browser',
	);

	if (browserIndex === -1) {
		return [];
	}

	return filteredArgv.slice(browserIndex + 1);
}
