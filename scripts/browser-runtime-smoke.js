import fsPromises from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const SUPPORTED_RUNTIME_TARGETS = new Set([
	'darwin-arm64',
	'darwin-x64',
	'linux-arm64',
	'linux-x64',
	'win32-arm64',
	'win32-x64',
]);

const SMOKE_MARKER = 'steel-browser-runtime-smoke-ok';

function runCommand(command, arguments_, options = {}) {
	const result = spawnSync(command, arguments_, {
		cwd: options.cwd,
		env: options.env,
		encoding: 'utf-8',
	});

	if (result.status !== 0) {
		const stderr = (result.stderr || '').trim();
		const stdout = (result.stdout || '').trim();
		throw new Error(
			`Command failed: ${command} ${arguments_.join(' ')}\n` +
				`exit=${result.status}\n` +
				`${stdout ? `stdout:\n${stdout}\n` : ''}` +
				`${stderr ? `stderr:\n${stderr}\n` : ''}`,
		);
	}

	return result;
}

function resolveNpmExecutable() {
	return process.platform === 'win32' ? 'npm.cmd' : 'npm';
}

function parseNpmPackOutput(output) {
	try {
		return JSON.parse(output);
	} catch {
		const start = output.indexOf('[');
		const end = output.lastIndexOf(']');
		if (start === -1 || end === -1) {
			throw new Error('Failed to parse npm pack --json output.');
		}

		return JSON.parse(output.slice(start, end + 1));
	}
}

function readManifestEntrypoint(manifestContent, runtimeTarget) {
	const parsed = JSON.parse(manifestContent);
	const entrypoint = parsed?.platforms?.[runtimeTarget]?.entrypoint;

	if (typeof entrypoint !== 'string' || !entrypoint.trim()) {
		throw new Error(
			`Runtime manifest does not define an entrypoint for ${runtimeTarget}.`,
		);
	}

	return entrypoint;
}

async function main() {
	const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(scriptDirectory, '..');
	const runtimeTarget = `${process.platform}-${process.arch}`;

	if (!SUPPORTED_RUNTIME_TARGETS.has(runtimeTarget)) {
		console.log(
			`[browser-runtime-smoke] Unsupported target ${runtimeTarget}; skipping smoke.`,
		);
		return;
	}

	const distEntry = path.join(projectRoot, 'dist/steel.js');
	try {
		await fsPromises.access(distEntry);
	} catch {
		throw new Error(
			'dist/steel.js is missing. Run `npm run build` before smoke tests.',
		);
	}

	const tempRoot = await fsPromises.mkdtemp(
		path.join(os.tmpdir(), 'steel-browser-runtime-smoke-'),
	);

	try {
		let sourceRoot = path.join(projectRoot, 'vendor/agent-browser');
		let entrypointRelative;
		let fixtureRuntime = false;
		const manifestPath = path.join(sourceRoot, 'runtime-manifest.json');
		let manifestContent = null;

		try {
			manifestContent = await fsPromises.readFile(manifestPath, 'utf-8');
		} catch (error) {
			if (error.code !== 'ENOENT') {
				throw error;
			}
		}

		if (manifestContent) {
			entrypointRelative = readManifestEntrypoint(
				manifestContent,
				runtimeTarget,
			);
		} else {
			fixtureRuntime = true;
			sourceRoot = path.join(tempRoot, 'vendor/agent-browser');
			entrypointRelative = `runtimes/${runtimeTarget}/cli.js`;
			const entrypointPath = path.join(sourceRoot, entrypointRelative);

			await fsPromises.mkdir(path.dirname(entrypointPath), {recursive: true});
			await fsPromises.writeFile(
				entrypointPath,
				[
					'const args = process.argv.slice(2);',
					`console.log('${SMOKE_MARKER} ' + args.join(' '));`,
					'process.exit(0);',
					'',
				].join('\n'),
				'utf-8',
			);

			await fsPromises.writeFile(
				path.join(sourceRoot, 'runtime-manifest.json'),
				JSON.stringify(
					{
						schemaVersion: 1,
						runtimeVersion: 'smoke-fixture',
						platforms: {
							[runtimeTarget]: {
								entrypoint: entrypointRelative,
							},
						},
					},
					null,
					2,
				),
				'utf-8',
			);
		}

		runCommand(process.execPath, ['scripts/package-browser-runtime.js'], {
			cwd: projectRoot,
			env: {
				...process.env,
				STEEL_BROWSER_RUNTIME_SOURCE: sourceRoot,
				STEEL_BROWSER_RUNTIME_OUTPUT: path.join(
					projectRoot,
					'dist/vendor/agent-browser',
				),
			},
		});

		const cliResult = runCommand(
			process.execPath,
			['dist/steel.js', 'browser', 'open', '--help'],
			{
				cwd: projectRoot,
				env: {
					...process.env,
					STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
				},
			},
		);

		const commandOutput = `${cliResult.stdout || ''}${cliResult.stderr || ''}`;
		if (
			fixtureRuntime &&
			!commandOutput.includes(`${SMOKE_MARKER} open --help`)
		) {
			throw new Error(
				'Packaged runtime did not execute for `steel browser open --help`.',
			);
		}

		const npmPackResult = runCommand(
			resolveNpmExecutable(),
			['pack', '--dry-run', '--json', '--ignore-scripts'],
			{
				cwd: projectRoot,
				env: {
					...process.env,
					npm_config_cache: path.join(tempRoot, '.npm-cache'),
				},
			},
		);
		const packEntries = parseNpmPackOutput(npmPackResult.stdout || '');
		const packedFiles = new Set(
			Array.isArray(packEntries) && packEntries[0]?.files
				? packEntries[0].files.map(file => file.path)
				: [],
		);

		const expectedManifestPath =
			'dist/vendor/agent-browser/runtime-manifest.json';
		const expectedEntrypointPath = `dist/vendor/agent-browser/${entrypointRelative}`;

		if (!packedFiles.has(expectedManifestPath)) {
			throw new Error(`npm pack output missing ${expectedManifestPath}.`);
		}

		if (!packedFiles.has(expectedEntrypointPath)) {
			throw new Error(`npm pack output missing ${expectedEntrypointPath}.`);
		}

		console.log(
			`[browser-runtime-smoke] Passed on ${runtimeTarget} with packaged runtime artifact checks.`,
		);
	} finally {
		await fsPromises.rm(tempRoot, {recursive: true, force: true});
	}
}

await main();
