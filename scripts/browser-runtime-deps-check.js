import fsPromises from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

async function readJsonFile(filePath) {
	const contents = await fsPromises.readFile(filePath, 'utf-8');
	return JSON.parse(contents);
}

function hasDependency(manifestDependencies, dependencyName) {
	return Boolean(
		manifestDependencies &&
			typeof manifestDependencies === 'object' &&
			manifestDependencies[dependencyName],
	);
}

async function main() {
	const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(scriptDirectory, '..');

	const packageManifestPath = path.join(projectRoot, 'package.json');
	const packageLockPath = path.join(projectRoot, 'package-lock.json');

	const packageManifest = await readJsonFile(packageManifestPath);
	const packageLock = await readJsonFile(packageLockPath);

	const dependencies = packageManifest.dependencies || {};
	if (hasDependency(dependencies, 'playwright')) {
		throw new Error(
			'`playwright` dependency is not allowed for Steel browser runtime. Use `playwright-core` only.',
		);
	}

	if (!hasDependency(dependencies, 'playwright-core')) {
		throw new Error(
			'`playwright-core` dependency is required for the vendored browser daemon runtime.',
		);
	}

	const lockRootDependencies = packageLock?.packages?.['']?.dependencies || {};
	if (hasDependency(lockRootDependencies, 'playwright')) {
		throw new Error(
			'package-lock root dependencies include `playwright`. Keep lockfile aligned to `playwright-core` only.',
		);
	}

	if (packageLock?.packages?.['node_modules/playwright']) {
		throw new Error(
			'package-lock contains `node_modules/playwright`. Remove it and keep `playwright-core` only.',
		);
	}

	console.log(
		'[browser-runtime-deps] OK: using playwright-core without playwright.',
	);
}

await main();
