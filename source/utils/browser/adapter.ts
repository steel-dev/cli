import {spawn} from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import {resolveBrowserAuth} from './auth.js';
import {BrowserAdapterError} from './errors.js';
import {bootstrapBrowserPassthroughArgv} from './lifecycle.js';
import {resolveVendoredRuntimePath} from './runtime.js';

export {resolveBrowserAuth} from './auth.js';
export {BrowserAdapterError} from './errors.js';

export type BrowserRuntimeCommand = {
	command: string;
	args: string[];
	source: 'env' | 'vendored' | 'path';
};

function hasBrowserAttachFlag(browserArgv: string[]): boolean {
	return browserArgv.some(argument => {
		return (
			argument === '--auto-connect' ||
			argument.startsWith('--auto-connect=') ||
			argument === '--cdp' ||
			argument.startsWith('--cdp=')
		);
	});
}

function isNodeScript(binaryPath: string): boolean {
	return (
		binaryPath.endsWith('.js') ||
		binaryPath.endsWith('.mjs') ||
		binaryPath.endsWith('.cjs')
	);
}

function toRuntimeCommand(
	runtimePath: string,
	source: BrowserRuntimeCommand['source'],
): BrowserRuntimeCommand {
	if (isNodeScript(runtimePath)) {
		return {
			command: process.execPath,
			args: [runtimePath],
			source,
		};
	}

	return {
		command: runtimePath,
		args: [],
		source,
	};
}

export function requiresBrowserAuth(browserArgv: string[]): boolean {
	if (browserArgv.includes('--help') || browserArgv.includes('-h')) {
		return false;
	}

	if (browserArgv.includes('--local')) {
		return false;
	}

	return !hasBrowserAttachFlag(browserArgv);
}

export function resolveBrowserRuntime(
	environment: NodeJS.ProcessEnv = process.env,
): BrowserRuntimeCommand {
	const configuredRuntime = environment.STEEL_BROWSER_RUNTIME_BIN?.trim();
	if (configuredRuntime) {
		return toRuntimeCommand(configuredRuntime, 'env');
	}

	const vendoredRuntimePath = resolveVendoredRuntimePath();
	if (vendoredRuntimePath) {
		return toRuntimeCommand(vendoredRuntimePath, 'vendored');
	}

	return {
		command: 'agent-browser',
		args: [],
		source: 'path',
	};
}

function runRuntime(
	runtime: BrowserRuntimeCommand,
	browserArgv: string[],
	environment: NodeJS.ProcessEnv,
): Promise<number> {
	return new Promise((resolve, reject) => {
		const runtimeProcess = spawn(
			runtime.command,
			[...runtime.args, ...browserArgv],
			{
				stdio: 'inherit',
				env: environment,
			},
		);

		runtimeProcess.once('error', error => {
			const errorCode = (error as NodeJS.ErrnoException).code;
			if (errorCode === 'ENOENT') {
				reject(
					new BrowserAdapterError(
						'RUNTIME_NOT_FOUND',
						`Browser runtime not found: ${runtime.command}`,
						error,
					),
				);
				return;
			}

			reject(
				new BrowserAdapterError(
					'SPAWN_ERROR',
					`Failed to execute browser runtime: ${runtime.command}`,
					error,
				),
			);
		});

		runtimeProcess.once('close', code => {
			resolve(code ?? 1);
		});
	});
}

function resolveVendoredRuntimeHome(
	runtime: BrowserRuntimeCommand,
): string | null {
	if (runtime.source !== 'vendored') {
		return null;
	}

	const runtimeEntrypoint = path.resolve(runtime.args[0] || runtime.command);
	let cursor = path.dirname(runtimeEntrypoint);

	while (true) {
		if (path.basename(cursor) === 'agent-browser') {
			const daemonEntrypoint = path.join(cursor, 'dist', 'daemon.js');
			return fs.existsSync(daemonEntrypoint) ? cursor : null;
		}

		const parent = path.dirname(cursor);
		if (parent === cursor) {
			return null;
		}

		cursor = parent;
	}
}

export async function runBrowserPassthrough(
	browserArgv: string[],
	environment: NodeJS.ProcessEnv = process.env,
): Promise<number> {
	if (browserArgv.length === 0) {
		throw new BrowserAdapterError(
			'INVALID_BROWSER_ARGS',
			'No browser command provided for passthrough execution.',
		);
	}

	const passthroughArgv = await bootstrapBrowserPassthroughArgv(
		browserArgv,
		environment,
	);
	const auth = resolveBrowserAuth(environment);

	const runtime = resolveBrowserRuntime(environment);
	const runtimeEnvironment = {...environment};

	if (!runtimeEnvironment.STEEL_API_KEY && auth.apiKey) {
		runtimeEnvironment.STEEL_API_KEY = auth.apiKey;
	}

	if (!runtimeEnvironment.AGENT_BROWSER_HOME) {
		const vendoredRuntimeHome = resolveVendoredRuntimeHome(runtime);
		if (vendoredRuntimeHome) {
			runtimeEnvironment.AGENT_BROWSER_HOME = vendoredRuntimeHome;
		}
	}

	return runRuntime(runtime, passthroughArgv, runtimeEnvironment);
}
