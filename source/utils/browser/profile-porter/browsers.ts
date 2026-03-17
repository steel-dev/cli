import * as os from 'node:os';
import * as path from 'node:path';
import type {BrowserDescriptor, BrowserId, OsPlatform} from './types.js';

export const SUPPORTED_BROWSERS: Record<BrowserId, BrowserDescriptor> = {
	chrome: {
		id: 'chrome',
		displayName: 'Google Chrome',
		profileBaseDirs: {
			darwin: 'Google/Chrome',
			win32: 'Google/Chrome/User Data',
			linux: 'google-chrome',
		},
		processNames: {
			darwin: ['Google Chrome'],
			win32: ['chrome.exe'],
			linux: ['chrome', 'google-chrome'],
		},
		keychainService: {
			darwin: 'Chrome Safe Storage',
			linux: 'chrome',
		},
	},
	edge: {
		id: 'edge',
		displayName: 'Microsoft Edge',
		profileBaseDirs: {
			darwin: 'Microsoft Edge',
			win32: 'Microsoft/Edge/User Data',
			linux: 'microsoft-edge',
		},
		processNames: {
			darwin: ['Microsoft Edge'],
			win32: ['msedge.exe'],
			linux: ['msedge', 'microsoft-edge'],
		},
		keychainService: {
			darwin: 'Microsoft Edge Safe Storage',
			linux: 'chromium',
		},
	},
	brave: {
		id: 'brave',
		displayName: 'Brave Browser',
		profileBaseDirs: {
			darwin: 'BraveSoftware/Brave-Browser',
			win32: 'BraveSoftware/Brave-Browser/User Data',
			linux: 'BraveSoftware/Brave-Browser',
		},
		processNames: {
			darwin: ['Brave Browser'],
			win32: ['brave.exe'],
			linux: ['brave', 'brave-browser'],
		},
		keychainService: {
			darwin: 'Brave Safe Storage',
			linux: 'brave',
		},
	},
	arc: {
		id: 'arc',
		displayName: 'Arc',
		profileBaseDirs: {
			darwin: 'Arc/User Data',
		},
		processNames: {
			darwin: ['Arc'],
		},
		keychainService: {
			darwin: 'Arc Safe Storage',
		},
	},
	opera: {
		id: 'opera',
		displayName: 'Opera',
		profileBaseDirs: {
			darwin: 'com.operasoftware.Opera',
			win32: 'Opera Software/Opera Stable',
			linux: 'opera',
		},
		processNames: {
			darwin: ['Opera'],
			win32: ['opera.exe'],
			linux: ['opera'],
		},
		keychainService: {
			darwin: 'Opera Safe Storage',
			linux: 'opera',
		},
	},
	vivaldi: {
		id: 'vivaldi',
		displayName: 'Vivaldi',
		profileBaseDirs: {
			darwin: 'Vivaldi',
			win32: 'Vivaldi/User Data',
			linux: 'vivaldi',
		},
		processNames: {
			darwin: ['Vivaldi'],
			win32: ['vivaldi.exe'],
			linux: ['vivaldi'],
		},
		keychainService: {
			darwin: 'Vivaldi Safe Storage',
			linux: 'vivaldi',
		},
	},
};

export function getBrowserDescriptor(id: BrowserId): BrowserDescriptor {
	const descriptor = SUPPORTED_BROWSERS[id];
	if (!descriptor) {
		throw new Error(`Unknown browser: ${id}`);
	}
	return descriptor;
}

/**
 * Resolves the absolute profile base directory for a browser on the current OS.
 * Returns `null` if the browser is not supported on this platform.
 */
export function getProfileBaseDir(
	browser: BrowserDescriptor,
	platform: OsPlatform = process.platform as OsPlatform,
): string | null {
	const relativeDir = browser.profileBaseDirs[platform];
	if (!relativeDir) return null;

	switch (platform) {
		case 'darwin':
			return path.join(
				os.homedir(),
				'Library',
				'Application Support',
				relativeDir,
			);
		case 'win32':
			return path.join(
				process.env['LOCALAPPDATA'] ||
					path.join(os.homedir(), 'AppData', 'Local'),
				relativeDir,
			);
		case 'linux':
			return path.join(os.homedir(), '.config', relativeDir);
		default:
			return null;
	}
}
