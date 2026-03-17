import * as fs from 'node:fs';
import * as path from 'node:path';
import {exec} from 'node:child_process';
import {deriveKey, deriveTargetKey, TARGET_ALGORITHM} from './crypto.js';
import {getProfileBaseDir} from './browsers.js';
import type {BrowserDescriptor, CryptoAlgorithm, KeyProvider} from './types.js';

const CBC_IV = Buffer.alloc(16, 0x20);

function execAsync(command: string): Promise<string> {
	return new Promise((resolve, reject) => {
		exec(command, {encoding: 'utf-8'}, (error, stdout) => {
			if (error) {
				reject(error);
			} else {
				resolve(stdout.trim());
			}
		});
	});
}

// ---------------------------------------------------------------------------
// macOS — Keychain
// ---------------------------------------------------------------------------

function createMacOSKeyProvider(browser: BrowserDescriptor): KeyProvider {
	const service = browser.keychainService?.darwin;
	if (!service) {
		throw new Error(
			`Browser "${browser.displayName}" has no macOS Keychain service configured.`,
		);
	}

	const sourceAlgorithm: CryptoAlgorithm = {type: 'aes-128-cbc', iv: CBC_IV};

	return {
		async getKey() {
			const passphrase = await execAsync(
				`security find-generic-password -w -s "${service}"`,
			);
			return deriveKey(passphrase, 1003);
		},
		getTargetKey: deriveTargetKey,
		sourceAlgorithm,
		targetAlgorithm: TARGET_ALGORITHM,
	};
}

// ---------------------------------------------------------------------------
// Linux — gnome-keyring / kwallet / fallback "peanuts"
// ---------------------------------------------------------------------------

function createLinuxKeyProvider(browser: BrowserDescriptor): KeyProvider {
	const appName = browser.keychainService?.linux;
	const sourceAlgorithm: CryptoAlgorithm = {type: 'aes-128-cbc', iv: CBC_IV};

	return {
		async getKey() {
			if (appName) {
				try {
					const passphrase = await execAsync(
						`secret-tool lookup application ${appName}`,
					);
					if (passphrase) {
						return deriveKey(passphrase, 1);
					}
				} catch {
					// Fall through to "peanuts" fallback
				}
			}

			// Chromium default when no keyring is available
			return deriveKey('peanuts', 1);
		},
		getTargetKey: deriveTargetKey,
		sourceAlgorithm,
		targetAlgorithm: TARGET_ALGORITHM,
	};
}

// ---------------------------------------------------------------------------
// Windows — DPAPI via PowerShell
// ---------------------------------------------------------------------------

function createWindowsKeyProvider(browser: BrowserDescriptor): KeyProvider {
	const sourceAlgorithm: CryptoAlgorithm = {
		type: 'aes-256-gcm',
		nonceLength: 12,
	};

	return {
		async getKey() {
			const baseDir = getProfileBaseDir(browser, 'win32');
			if (!baseDir) {
				throw new Error(
					`Browser "${browser.displayName}" is not supported on Windows.`,
				);
			}

			const localStatePath = path.join(baseDir, 'Local State');
			if (!fs.existsSync(localStatePath)) {
				throw new Error(`Could not find Local State file at ${localStatePath}`);
			}

			const localState = JSON.parse(
				fs.readFileSync(localStatePath, 'utf-8'),
			) as {os_crypt?: {encrypted_key?: string}};

			const encryptedKeyB64 = localState?.os_crypt?.encrypted_key;
			if (!encryptedKeyB64) {
				throw new Error(
					'Local State file does not contain os_crypt.encrypted_key',
				);
			}

			const encryptedKeyRaw = Buffer.from(encryptedKeyB64, 'base64');

			// Strip the "DPAPI" prefix (5 bytes)
			const dpapiPrefix = encryptedKeyRaw.slice(0, 5).toString('ascii');
			if (dpapiPrefix !== 'DPAPI') {
				throw new Error(
					`Unexpected encrypted key prefix: "${dpapiPrefix}" (expected "DPAPI")`,
				);
			}

			const dpapiBlob = encryptedKeyRaw.slice(5).toString('base64');

			const psScript = [
				'Add-Type -AssemblyName System.Security',
				`$blob = [Convert]::FromBase64String('${dpapiBlob}')`,
				'$plain = [Security.Cryptography.ProtectedData]::Unprotect($blob, $null, [Security.Cryptography.DataProtectionScope]::CurrentUser)',
				'[Convert]::ToBase64String($plain)',
			].join('; ');

			const result = await execAsync(
				`powershell -NoProfile -NonInteractive -Command "${psScript.replace(/"/g, '\\"')}"`,
			);

			return Buffer.from(result, 'base64');
		},
		getTargetKey: deriveTargetKey,
		sourceAlgorithm,
		targetAlgorithm: TARGET_ALGORITHM,
	};
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

export function createKeyProvider(
	browser: BrowserDescriptor,
	platform: NodeJS.Platform = process.platform,
): KeyProvider {
	switch (platform) {
		case 'darwin':
			return createMacOSKeyProvider(browser);
		case 'linux':
			return createLinuxKeyProvider(browser);
		case 'win32':
			return createWindowsKeyProvider(browser);
		default:
			throw new Error(
				`Unsupported platform for cookie decryption: ${platform}`,
			);
	}
}
