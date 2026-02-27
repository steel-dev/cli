import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	getBrowserRuntimeTarget,
	getVendoredRuntimeSearchRoots,
	resolveVendoredRuntimeFromManifest,
	resolveVendoredRuntimePath,
} from '../../source/utils/browser/runtime';

function writeRuntimeFile(rootPath: string, relativeFilePath: string): string {
	const runtimePath = path.join(rootPath, relativeFilePath);
	fs.mkdirSync(path.dirname(runtimePath), {recursive: true});
	fs.writeFileSync(runtimePath, 'console.log("runtime");\n', 'utf-8');
	return runtimePath;
}

describe('browser runtime target resolution', () => {
	test('maps supported platform and architecture combinations', () => {
		expect(getBrowserRuntimeTarget('linux', 'x64')).toBe('linux-x64');
		expect(getBrowserRuntimeTarget('darwin', 'arm64')).toBe('darwin-arm64');
		expect(getBrowserRuntimeTarget('win32', 'x64')).toBe('win32-x64');
	});

	test('returns null for unsupported combinations', () => {
		expect(getBrowserRuntimeTarget('freebsd', 'x64')).toBeNull();
		expect(getBrowserRuntimeTarget('linux', 'ppc64')).toBeNull();
	});
});

describe('vendored browser runtime lookup', () => {
	let temporaryRoot: string;

	beforeEach(() => {
		temporaryRoot = fs.mkdtempSync(
			path.join(os.tmpdir(), 'steel-browser-runtime-test-'),
		);
	});

	afterEach(() => {
		fs.rmSync(temporaryRoot, {recursive: true, force: true});
	});

	test('resolves manifest-based runtime for current target', () => {
		const runtimePath = writeRuntimeFile(
			temporaryRoot,
			'vendor/agent-browser/runtimes/linux-x64/cli.js',
		);

		fs.writeFileSync(
			path.join(temporaryRoot, 'vendor/agent-browser/runtime-manifest.json'),
			JSON.stringify(
				{
					schemaVersion: 1,
					runtimeVersion: 'test',
					platforms: {
						'linux-x64': {
							entrypoint: 'runtimes/linux-x64/cli.js',
						},
					},
				},
				null,
				2,
			),
			'utf-8',
		);

		expect(
			resolveVendoredRuntimeFromManifest([temporaryRoot], 'linux', 'x64'),
		).toBe(runtimePath);
	});

	test('ignores invalid manifest entries', () => {
		fs.mkdirSync(path.join(temporaryRoot, 'vendor/agent-browser'), {
			recursive: true,
		});

		fs.writeFileSync(
			path.join(temporaryRoot, 'vendor/agent-browser/runtime-manifest.json'),
			JSON.stringify(
				{
					schemaVersion: 1,
					runtimeVersion: 'test',
					platforms: {
						'linux-x64': {
							entrypoint: '../outside.js',
						},
					},
				},
				null,
				2,
			),
			'utf-8',
		);

		expect(
			resolveVendoredRuntimeFromManifest([temporaryRoot], 'linux', 'x64'),
		).toBeNull();
	});

	test('falls back to legacy vendored runtime path when manifest is absent', () => {
		const legacyPath = writeRuntimeFile(
			temporaryRoot,
			'dist/vendor/agent-browser/cli.js',
		);

		expect(resolveVendoredRuntimePath([temporaryRoot], 'linux', 'x64')).toBe(
			legacyPath,
		);
	});

	test('includes realpath-derived root for symlinked executable paths', () => {
		const workingDirectory = path.join(temporaryRoot, 'workspace');
		const installRoot = path.join(
			temporaryRoot,
			'node_modules',
			'@steel-dev',
			'cli',
		);
		const symlinkPath = path.join(temporaryRoot, 'bin', 'steel');
		const resolvedEntrypoint = path.join(installRoot, 'dist', 'steel.js');

		fs.mkdirSync(workingDirectory, {recursive: true});
		fs.mkdirSync(path.dirname(resolvedEntrypoint), {recursive: true});
		fs.mkdirSync(path.dirname(symlinkPath), {recursive: true});
		fs.writeFileSync(resolvedEntrypoint, '#!/usr/bin/env node\n', 'utf-8');

		try {
			fs.symlinkSync(resolvedEntrypoint, symlinkPath);
		} catch (error) {
			if ((error as NodeJS.ErrnoException).code === 'EPERM') {
				return;
			}

			throw error;
		}

		const searchRoots = getVendoredRuntimeSearchRoots(
			workingDirectory,
			symlinkPath,
		);

		expect(searchRoots).toContain(workingDirectory);
		expect(searchRoots).toContain(path.resolve(temporaryRoot));
		expect(searchRoots).toContain(fs.realpathSync(installRoot));
	});
});
