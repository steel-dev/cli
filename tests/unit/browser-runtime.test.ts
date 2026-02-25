import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	getBrowserRuntimeTarget,
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
});
