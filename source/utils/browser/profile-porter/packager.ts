import * as fs from 'node:fs';
import * as path from 'node:path';
import {zipSync} from 'fflate';
import {makeReencryptedCookiesBuffer} from './crypto.js';
import {collectFiles, INCLUDE_ENTRIES} from './discovery.js';
import {createKeyProvider} from './key-providers.js';
import {getBrowserDescriptor, getProfileBaseDir} from './browsers.js';
import type {
	KeyProvider,
	PackageProfileOptions,
	PackageResult,
} from './types.js';

export async function packageProfile(
	options: PackageProfileOptions,
): Promise<PackageResult> {
	const {profileDir, keyProvider, onProgress, onKeyPrompt} = options;

	if (!fs.existsSync(path.join(profileDir, 'Cookies'))) {
		throw new Error(`Profile not found at ${profileDir} (no Cookies file)`);
	}

	onKeyPrompt?.();
	// Let Ink render the message before the OS key dialog appears
	await new Promise(resolve => setTimeout(resolve, 100));

	const sourceKey = await keyProvider.getKey();
	const targetKey = keyProvider.getTargetKey();

	const files: Record<string, Uint8Array> = {};
	let cookiesReencrypted = 0;

	for (const entry of INCLUDE_ENTRIES) {
		const fullPath = path.join(profileDir, entry);
		if (!fs.existsSync(fullPath)) continue;

		const stat = fs.statSync(fullPath);

		if (stat.isFile()) {
			if (entry === 'Cookies') {
				onProgress?.('Re-encrypting Cookies...');
				const {buffer, converted} = makeReencryptedCookiesBuffer(
					fullPath,
					sourceKey,
					targetKey,
					keyProvider.sourceAlgorithm,
					keyProvider.targetAlgorithm,
				);
				files[`Default/${entry}`] = new Uint8Array(buffer);
				cookiesReencrypted = converted;
			} else {
				files[`Default/${entry}`] = new Uint8Array(fs.readFileSync(fullPath));
			}
		} else if (stat.isDirectory()) {
			onProgress?.(`Collecting ${entry}/...`);
			const dirFiles = collectFiles(fullPath, fullPath, {});
			for (const [relPath, data] of Object.entries(dirFiles)) {
				files[`Default/${entry}/${relPath}`] = data;
			}
		}
	}

	onProgress?.('Zipping...');
	const zipped = zipSync(files, {level: 6});
	return {zipBuffer: Buffer.from(zipped), cookiesReencrypted};
}

/**
 * @deprecated Use `packageProfile()` with explicit browser and key provider.
 */
export async function packageChromeProfile(
	chromeProfile: string,
	onProgress?: (msg: string) => void,
	onKeychainPrompt?: () => void,
): Promise<PackageResult> {
	const browser = getBrowserDescriptor('chrome');
	const baseDir = getProfileBaseDir(browser);
	if (!baseDir) {
		throw new Error('Chrome is not supported on this platform.');
	}

	const keyProvider: KeyProvider = createKeyProvider(browser);

	return packageProfile({
		profileDir: path.join(baseDir, chromeProfile),
		keyProvider,
		onProgress,
		onKeyPrompt: onKeychainPrompt,
	});
}
