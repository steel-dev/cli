import * as fs from 'node:fs';
import * as path from 'node:path';

export type BrowserRuntimeManifest = {
	schemaVersion: number;
	runtimeVersion: string;
	platforms: Record<
		string,
		{
			entrypoint: string;
		}
	>;
};

const SUPPORTED_RUNTIME_TARGETS = new Set([
	'darwin-arm64',
	'darwin-x64',
	'linux-arm64',
	'linux-x64',
	'win32-arm64',
	'win32-x64',
]);

function isObject(value: unknown): value is Record<string, unknown> {
	return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function isSafeRelativePath(value: string): boolean {
	if (!value || path.isAbsolute(value)) {
		return false;
	}

	const normalizedPath = value.split('\\').join('/');
	return !normalizedPath.split('/').includes('..');
}

function parseManifest(jsonContents: string): BrowserRuntimeManifest | null {
	try {
		const parsed = JSON.parse(jsonContents) as Record<string, unknown>;
		const platformsValue = parsed.platforms;

		if (!isObject(platformsValue)) {
			return null;
		}

		const platforms: BrowserRuntimeManifest['platforms'] = {};
		for (const [target, config] of Object.entries(platformsValue)) {
			if (!isObject(config)) {
				return null;
			}

			const entrypoint = config.entrypoint;
			if (typeof entrypoint !== 'string' || !isSafeRelativePath(entrypoint)) {
				return null;
			}

			platforms[target] = {entrypoint};
		}

		return {
			schemaVersion:
				typeof parsed.schemaVersion === 'number' ? parsed.schemaVersion : 1,
			runtimeVersion:
				typeof parsed.runtimeVersion === 'string'
					? parsed.runtimeVersion
					: 'unknown',
			platforms,
		};
	} catch {
		return null;
	}
}

export function getBrowserRuntimeTarget(
	platform: NodeJS.Platform = process.platform,
	architecture: NodeJS.Architecture = process.arch,
): string | null {
	const runtimeTarget = `${platform}-${architecture}`;
	if (!SUPPORTED_RUNTIME_TARGETS.has(runtimeTarget)) {
		return null;
	}

	return runtimeTarget;
}

export function getVendoredRuntimeSearchRoots(
	cwd = process.cwd(),
	processArgv1 = process.argv[1],
): string[] {
	const searchRoots = new Set<string>([cwd]);

	if (processArgv1) {
		searchRoots.add(path.resolve(path.dirname(processArgv1), '..'));
	}

	return [...searchRoots];
}

function getRuntimeManifestCandidates(searchRoot: string): string[] {
	return [
		path.join(searchRoot, 'dist/vendor/agent-browser/runtime-manifest.json'),
		path.join(searchRoot, 'vendor/agent-browser/runtime-manifest.json'),
	];
}

function getLegacyVendoredRuntimeCandidates(
	searchRoot: string,
	runtimeTarget: string | null,
): string[] {
	return [
		path.join(searchRoot, 'dist/vendor/agent-browser/cli.js'),
		path.join(searchRoot, 'dist/vendor/agent-browser/index.js'),
		...(runtimeTarget
			? [
					path.join(
						searchRoot,
						'dist/vendor/agent-browser/runtimes',
						runtimeTarget,
						'cli.js',
					),
					path.join(
						searchRoot,
						'vendor/agent-browser/runtimes',
						runtimeTarget,
						'cli.js',
					),
					path.join(
						searchRoot,
						'third_party/agent-browser/runtimes',
						runtimeTarget,
						'cli.js',
					),
				]
			: []),
		path.join(searchRoot, 'vendor/agent-browser/cli.js'),
		path.join(searchRoot, 'third_party/agent-browser/cli.js'),
	];
}

export function resolveVendoredRuntimeFromManifest(
	searchRoots = getVendoredRuntimeSearchRoots(),
	platform: NodeJS.Platform = process.platform,
	architecture: NodeJS.Architecture = process.arch,
): string | null {
	const runtimeTarget = getBrowserRuntimeTarget(platform, architecture);
	if (!runtimeTarget) {
		return null;
	}

	for (const searchRoot of searchRoots) {
		for (const manifestPath of getRuntimeManifestCandidates(searchRoot)) {
			if (!fs.existsSync(manifestPath)) {
				continue;
			}

			const manifest = parseManifest(fs.readFileSync(manifestPath, 'utf-8'));
			if (!manifest) {
				continue;
			}

			const platformConfig = manifest.platforms[runtimeTarget];
			if (!platformConfig) {
				continue;
			}

			const runtimePath = path.resolve(
				path.dirname(manifestPath),
				platformConfig.entrypoint,
			);
			if (fs.existsSync(runtimePath)) {
				return runtimePath;
			}
		}
	}

	return null;
}

export function resolveVendoredRuntimePath(
	searchRoots = getVendoredRuntimeSearchRoots(),
	platform: NodeJS.Platform = process.platform,
	architecture: NodeJS.Architecture = process.arch,
): string | null {
	const runtimeFromManifest = resolveVendoredRuntimeFromManifest(
		searchRoots,
		platform,
		architecture,
	);
	if (runtimeFromManifest) {
		return runtimeFromManifest;
	}

	const runtimeTarget = getBrowserRuntimeTarget(platform, architecture);
	for (const searchRoot of searchRoots) {
		for (const candidatePath of getLegacyVendoredRuntimeCandidates(
			searchRoot,
			runtimeTarget,
		)) {
			if (fs.existsSync(candidatePath)) {
				return candidatePath;
			}
		}
	}

	return null;
}
