import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {spawnSync} from 'node:child_process';

const DEFAULT_CONFIG_DIR = path.join(os.homedir(), '.config', 'steel');
const DEFAULT_REPO_URL = 'https://github.com/steel-dev/steel-browser.git';

export type CommandRunOptions = {
	cwd?: string;
	env?: NodeJS.ProcessEnv;
	stdio?: 'ignore' | 'inherit';
};

export type CommandRunResult = {
	status: number | null;
	error?: NodeJS.ErrnoException | Error;
};

export type CommandRunner = (
	command: string,
	args: string[],
	options?: CommandRunOptions,
) => CommandRunResult;

export type ComposeCommand = {
	command: string;
	baseArgs: string[];
};

type LocalDevContext = {
	configDirectory?: string;
	repoUrl?: string;
	runner?: CommandRunner;
	environment?: NodeJS.ProcessEnv;
};

type InstallLocalBrowserRuntimeOptions = LocalDevContext & {
	verbose?: boolean;
};

type StartLocalBrowserRuntimeOptions = LocalDevContext & {
	port?: number;
	verbose?: boolean;
	skipDockerCheck?: boolean;
};

type StopLocalBrowserRuntimeOptions = LocalDevContext & {
	verbose?: boolean;
};

function defaultRunner(
	command: string,
	args: string[],
	options: CommandRunOptions = {},
): CommandRunResult {
	const result = spawnSync(command, args, {
		cwd: options.cwd,
		env: options.env,
		stdio: options.stdio || 'ignore',
	});

	return {
		status: result.status,
		error: result.error as NodeJS.ErrnoException | undefined,
	};
}

function isSuccess(result: CommandRunResult): boolean {
	return result.status === 0;
}

export function getLocalBrowserRepoPath(
	configDirectory = DEFAULT_CONFIG_DIR,
	repoUrl = DEFAULT_REPO_URL,
): string {
	const repoName = path.basename(repoUrl, '.git');
	return path.join(configDirectory, repoName);
}

export function deriveLocalApiPort(
	environment: NodeJS.ProcessEnv = process.env,
	explicitPort?: number,
): string {
	if (explicitPort && Number.isInteger(explicitPort) && explicitPort > 0) {
		return String(explicitPort);
	}

	const configuredApiUrl =
		environment.STEEL_BROWSER_API_URL?.trim() ||
		environment.STEEL_LOCAL_API_URL?.trim();
	if (!configuredApiUrl) {
		return '3000';
	}

	try {
		const parsedUrl = new URL(configuredApiUrl);
		if (parsedUrl.port) {
			return parsedUrl.port;
		}

		if (parsedUrl.protocol === 'http:') {
			return '80';
		}

		if (parsedUrl.protocol === 'https:') {
			return '443';
		}
	} catch {
		const portMatch = configuredApiUrl.match(/:(\d+)(?:\/|$)/);
		if (portMatch?.[1]) {
			return portMatch[1];
		}
	}

	return '3000';
}

export function isDockerRunning(
	runner: CommandRunner = defaultRunner,
): boolean {
	const result = runner('docker', ['info'], {stdio: 'ignore'});
	return isSuccess(result);
}

export function resolveComposeCommand(
	runner: CommandRunner = defaultRunner,
): ComposeCommand | null {
	const dockerComposeV1 = runner('docker-compose', ['version'], {
		stdio: 'ignore',
	});
	if (isSuccess(dockerComposeV1)) {
		return {command: 'docker-compose', baseArgs: []};
	}

	const dockerComposeV2 = runner('docker', ['compose', 'version'], {
		stdio: 'ignore',
	});
	if (isSuccess(dockerComposeV2)) {
		return {command: 'docker', baseArgs: ['compose']};
	}

	return null;
}

function cloneLocalBrowserRepo(
	configDirectory: string,
	repoUrl: string,
	runner: CommandRunner,
	verbose = false,
): string {
	const repoPath = getLocalBrowserRepoPath(configDirectory, repoUrl);

	fs.mkdirSync(configDirectory, {recursive: true});
	const cloneResult = runner('git', ['clone', repoUrl], {
		cwd: configDirectory,
		stdio: verbose ? 'inherit' : 'ignore',
	});

	if (!isSuccess(cloneResult) || !fs.existsSync(repoPath)) {
		throw new Error(
			`Failed to clone local browser runtime repository (${repoUrl}).`,
		);
	}

	return repoPath;
}

export function installLocalBrowserRuntime(
	options: InstallLocalBrowserRuntimeOptions = {},
): {repoPath: string; repoUrl: string; installed: boolean} {
	const configDirectory = options.configDirectory || DEFAULT_CONFIG_DIR;
	const repoUrl = options.repoUrl || DEFAULT_REPO_URL;
	const runner = options.runner || defaultRunner;
	const repoPath = getLocalBrowserRepoPath(configDirectory, repoUrl);

	if (fs.existsSync(repoPath)) {
		return {
			repoPath,
			repoUrl,
			installed: false,
		};
	}

	const clonedRepoPath = cloneLocalBrowserRepo(
		configDirectory,
		repoUrl,
		runner,
		options.verbose,
	);

	return {
		repoPath: clonedRepoPath,
		repoUrl,
		installed: true,
	};
}

export function startLocalBrowserRuntime(
	options: StartLocalBrowserRuntimeOptions = {},
): {repoPath: string; apiPort: string; composeCommand: ComposeCommand} {
	const configDirectory = options.configDirectory || DEFAULT_CONFIG_DIR;
	const repoUrl = options.repoUrl || DEFAULT_REPO_URL;
	const environment = options.environment || process.env;
	const runner = options.runner || defaultRunner;
	const verbose = options.verbose || false;
	const repoPath = getLocalBrowserRepoPath(configDirectory, repoUrl);

	if (!fs.existsSync(repoPath)) {
		throw new Error(
			'Local browser runtime is not installed. Run `steel dev install` first.',
		);
	}

	if (!options.skipDockerCheck && !isDockerRunning(runner)) {
		throw new Error('Docker is not running. Start Docker and try again.');
	}

	const composeCommand = resolveComposeCommand(runner);
	if (!composeCommand) {
		throw new Error(
			'Could not find Docker Compose. Install `docker compose` (v2) or `docker-compose`.',
		);
	}

	const apiPort = deriveLocalApiPort(environment, options.port);
	const composeArgs = [
		...composeCommand.baseArgs,
		'-f',
		'docker-compose.yml',
		'up',
		'-d',
	];
	const composeResult = runner(composeCommand.command, composeArgs, {
		cwd: repoPath,
		env: {
			...environment,
			API_PORT: apiPort,
		},
		stdio: verbose ? 'inherit' : 'ignore',
	});

	if (!isSuccess(composeResult)) {
		throw new Error('Failed to start local Steel Browser runtime.');
	}

	return {
		repoPath,
		apiPort,
		composeCommand,
	};
}

export function stopLocalBrowserRuntime(
	options: StopLocalBrowserRuntimeOptions = {},
): {repoPath: string; composeCommand: ComposeCommand} {
	const configDirectory = options.configDirectory || DEFAULT_CONFIG_DIR;
	const repoUrl = options.repoUrl || DEFAULT_REPO_URL;
	const runner = options.runner || defaultRunner;
	const repoPath = getLocalBrowserRepoPath(configDirectory, repoUrl);

	if (!fs.existsSync(repoPath)) {
		throw new Error(
			'Local browser runtime is not installed. Run `steel dev install` first.',
		);
	}

	const composeCommand = resolveComposeCommand(runner);
	if (!composeCommand) {
		throw new Error(
			'Could not find Docker Compose. Install `docker compose` (v2) or `docker-compose`.',
		);
	}

	const composeArgs = [
		...composeCommand.baseArgs,
		'-f',
		'docker-compose.yml',
		'down',
	];
	const composeResult = runner(composeCommand.command, composeArgs, {
		cwd: repoPath,
		env: process.env,
		stdio: options.verbose ? 'inherit' : 'ignore',
	});

	if (!isSuccess(composeResult)) {
		throw new Error('Failed to stop local Steel Browser runtime.');
	}

	return {
		repoPath,
		composeCommand,
	};
}
