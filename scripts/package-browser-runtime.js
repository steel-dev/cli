import fs from 'node:fs';
import fsPromises from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const MANIFEST_FILE_NAME = 'runtime-manifest.json';

function isObject(value) {
	return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function isSafeRelativePath(value) {
	if (!value || path.isAbsolute(value)) {
		return false;
	}

	const normalizedPath = value.replaceAll('\\', '/');
	return !normalizedPath.split('/').includes('..');
}

function parseManifest(manifestContent) {
	const parsed = JSON.parse(manifestContent);
	if (!isObject(parsed) || !isObject(parsed.platforms)) {
		throw new Error(
			'Browser runtime manifest must contain a platforms object.',
		);
	}

	const platforms = {};
	for (const [target, config] of Object.entries(parsed.platforms)) {
		if (!isObject(config) || typeof config.entrypoint !== 'string') {
			throw new Error(
				`Browser runtime manifest target "${target}" is missing a valid entrypoint.`,
			);
		}

		if (!isSafeRelativePath(config.entrypoint)) {
			throw new Error(
				`Browser runtime manifest target "${target}" has an unsafe entrypoint path.`,
			);
		}

		platforms[target] = {entrypoint: config.entrypoint};
	}

	const shared = [];
	if (parsed.shared !== undefined) {
		if (!Array.isArray(parsed.shared)) {
			throw new Error(
				'Browser runtime manifest shared value must be an array.',
			);
		}

		for (const sharedPath of parsed.shared) {
			if (typeof sharedPath !== 'string' || !isSafeRelativePath(sharedPath)) {
				throw new Error(
					'Browser runtime manifest shared entries must be safe relative paths.',
				);
			}

			shared.push(sharedPath);
		}
	}

	return {
		schemaVersion:
			typeof parsed.schemaVersion === 'number' ? parsed.schemaVersion : 1,
		runtimeVersion:
			typeof parsed.runtimeVersion === 'string'
				? parsed.runtimeVersion
				: 'unknown',
		platforms,
		shared,
	};
}

async function ensurePathExists(pathToCheck) {
	try {
		await fsPromises.access(pathToCheck, fs.constants.F_OK);
		return true;
	} catch {
		return false;
	}
}

async function copyRelativePath(sourceRoot, outputRoot, relativePath) {
	const sourcePath = path.join(sourceRoot, relativePath);
	const outputPath = path.join(outputRoot, relativePath);
	const stats = await fsPromises.stat(sourcePath);

	await fsPromises.mkdir(path.dirname(outputPath), {recursive: true});

	if (stats.isDirectory()) {
		await fsPromises.cp(sourcePath, outputPath, {recursive: true});
		return;
	}

	await fsPromises.copyFile(sourcePath, outputPath);
}

async function packageBrowserRuntime() {
	const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(scriptDirectory, '..');

	const sourceRoot = path.resolve(
		process.env.STEEL_BROWSER_RUNTIME_SOURCE ||
			path.join(projectRoot, 'vendor/agent-browser'),
	);
	const outputRoot = path.resolve(
		process.env.STEEL_BROWSER_RUNTIME_OUTPUT ||
			path.join(projectRoot, 'dist/vendor/agent-browser'),
	);
	const sourceManifestPath = path.join(sourceRoot, MANIFEST_FILE_NAME);

	if (!(await ensurePathExists(sourceManifestPath))) {
		console.log(
			`[browser-runtime] No ${MANIFEST_FILE_NAME} found at ${sourceRoot}; skipping runtime packaging.`,
		);
		return;
	}

	const manifest = parseManifest(
		await fsPromises.readFile(sourceManifestPath, 'utf-8'),
	);

	const pathsToCopy = new Set(manifest.shared);
	for (const platformConfig of Object.values(manifest.platforms)) {
		const entrypointRelative = platformConfig.entrypoint;
		const entrypointDirectory = path.dirname(entrypointRelative);
		pathsToCopy.add(
			entrypointDirectory === '.' ? entrypointRelative : entrypointDirectory,
		);
	}

	for (const relativePath of pathsToCopy) {
		if (!(await ensurePathExists(path.join(sourceRoot, relativePath)))) {
			throw new Error(
				`Runtime manifest references missing path: ${relativePath}`,
			);
		}
	}

	await fsPromises.rm(outputRoot, {recursive: true, force: true});
	await fsPromises.mkdir(outputRoot, {recursive: true});

	await fsPromises.copyFile(
		sourceManifestPath,
		path.join(outputRoot, MANIFEST_FILE_NAME),
	);

	for (const relativePath of pathsToCopy) {
		await copyRelativePath(sourceRoot, outputRoot, relativePath);
	}

	console.log(
		`[browser-runtime] Packaged ${Object.keys(manifest.platforms).length} runtime target(s) to ${path.relative(projectRoot, outputRoot)}.`,
	);
}

await packageBrowserRuntime();
