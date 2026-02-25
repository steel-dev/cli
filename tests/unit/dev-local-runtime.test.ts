import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	deriveLocalApiPort,
	resolveComposeCommand,
	startLocalBrowserRuntime,
	stopLocalBrowserRuntime,
	type CommandRunOptions,
	type CommandRunResult,
	type CommandRunner,
} from '../../source/utils/dev/local';

type Invocation = {
	command: string;
	args: string[];
	options?: CommandRunOptions;
};

function createTempDirectory(): string {
	return fs.mkdtempSync(path.join(os.tmpdir(), 'steel-dev-local-runtime-'));
}

describe('local runtime port derivation', () => {
	test('prefers explicit port value', () => {
		expect(deriveLocalApiPort({}, 4123)).toBe('4123');
	});

	test('derives port from STEEL_API_URL', () => {
		expect(
			deriveLocalApiPort({STEEL_API_URL: 'http://localhost:3355/v1'}),
		).toBe('3355');
	});

	test('falls back to 3000 when API URL is missing or invalid', () => {
		expect(deriveLocalApiPort({})).toBe('3000');
		expect(deriveLocalApiPort({STEEL_API_URL: 'not-a-url'})).toBe('3000');
	});
});

describe('local runtime compose detection', () => {
	test('uses docker-compose when available', () => {
		const runner: CommandRunner = (command, args) => {
			if (command === 'docker-compose' && args[0] === 'version') {
				return {status: 0};
			}

			return {status: 1};
		};

		expect(resolveComposeCommand(runner)).toEqual({
			command: 'docker-compose',
			baseArgs: [],
		});
	});

	test('falls back to docker compose when docker-compose is unavailable', () => {
		const runner: CommandRunner = (command, args) => {
			if (command === 'docker-compose' && args[0] === 'version') {
				return {status: 1};
			}

			if (command === 'docker' && args.join(' ') === 'compose version') {
				return {status: 0};
			}

			return {status: 1};
		};

		expect(resolveComposeCommand(runner)).toEqual({
			command: 'docker',
			baseArgs: ['compose'],
		});
	});
});

describe('local runtime lifecycle', () => {
	test('starts runtime with clone + compose up', () => {
		const tempDirectory = createTempDirectory();
		const repoUrl = 'https://github.com/steel-dev/steel-browser.git';
		const expectedRepoPath = path.join(tempDirectory, 'steel-browser');
		const invocations: Invocation[] = [];

		const runner: CommandRunner = (
			command: string,
			args: string[],
			options?: CommandRunOptions,
		): CommandRunResult => {
			invocations.push({command, args, options});

			if (command === 'docker' && args[0] === 'info') {
				return {status: 0};
			}

			if (command === 'docker-compose' && args[0] === 'version') {
				return {status: 0};
			}

			if (command === 'git' && args[0] === 'clone') {
				fs.mkdirSync(expectedRepoPath, {recursive: true});
				return {status: 0};
			}

			if (
				command === 'docker-compose' &&
				args.join(' ') === '-f docker-compose.yml up -d'
			) {
				return {status: 0};
			}

			return {status: 1};
		};

		try {
			const result = startLocalBrowserRuntime({
				configDirectory: tempDirectory,
				repoUrl,
				runner,
				environment: {STEEL_API_URL: 'http://localhost:4567/v1'},
			});

			expect(result.repoPath).toBe(expectedRepoPath);
			expect(result.apiPort).toBe('4567');
			expect(result.composeCommand).toEqual({
				command: 'docker-compose',
				baseArgs: [],
			});

			const composeInvocation = invocations.find(
				invocation =>
					invocation.command === 'docker-compose' &&
					invocation.args.includes('up'),
			);
			expect(composeInvocation?.options?.cwd).toBe(expectedRepoPath);
			expect(composeInvocation?.options?.env?.API_PORT).toBe('4567');
		} finally {
			fs.rmSync(tempDirectory, {recursive: true, force: true});
		}
	});

	test('stops runtime with compose down', () => {
		const tempDirectory = createTempDirectory();
		const repoUrl = 'https://github.com/steel-dev/steel-browser.git';
		const repoPath = path.join(tempDirectory, 'steel-browser');
		const invocations: Invocation[] = [];

		fs.mkdirSync(repoPath, {recursive: true});

		const runner: CommandRunner = (
			command: string,
			args: string[],
			options?: CommandRunOptions,
		): CommandRunResult => {
			invocations.push({command, args, options});

			if (command === 'docker-compose' && args[0] === 'version') {
				return {status: 0};
			}

			if (
				command === 'docker-compose' &&
				args.join(' ') === '-f docker-compose.yml down'
			) {
				return {status: 0};
			}

			return {status: 1};
		};

		try {
			const result = stopLocalBrowserRuntime({
				configDirectory: tempDirectory,
				repoUrl,
				runner,
			});

			expect(result.repoPath).toBe(repoPath);
			const composeInvocation = invocations.find(
				invocation =>
					invocation.command === 'docker-compose' &&
					invocation.args.includes('down'),
			);
			expect(composeInvocation?.options?.cwd).toBe(repoPath);
		} finally {
			fs.rmSync(tempDirectory, {recursive: true, force: true});
		}
	});

	test('throws when stopping without a checked-out repository', () => {
		const tempDirectory = createTempDirectory();

		try {
			expect(() =>
				stopLocalBrowserRuntime({
					configDirectory: tempDirectory,
				}),
			).toThrow('No local browser runtime repository found');
		} finally {
			fs.rmSync(tempDirectory, {recursive: true, force: true});
		}
	});
});
