import * as fs from 'node:fs';
import * as path from 'node:path';
import {execSync} from 'node:child_process';
import {SUPPORTED_BROWSERS, getProfileBaseDir} from './browsers.js';
import type {
	BrowserDescriptor,
	BrowserId,
	BrowserProfile,
	OsPlatform,
} from './types.js';

const SKIP_NAMES = new Set([
	'LOCK',
	'SingletonLock',
	'SingletonCookie',
	'SingletonSocket',
]);

const SKIP_EXTS = new Set(['.log', '.pma']);

export {SKIP_NAMES, SKIP_EXTS};

export const INCLUDE_ENTRIES = [
	'Cookies',
	'Local Storage',
	'IndexedDB',
	'Preferences',
	'Bookmarks',
	'Favicons',
	'History',
	'Web Data',
];

export function getProfileDisplayName(
	baseDir: string,
	dirName: string,
): string {
	try {
		const prefsPath = path.join(baseDir, dirName, 'Preferences');
		const prefs = JSON.parse(fs.readFileSync(prefsPath, 'utf-8')) as {
			profile?: {name?: string};
			account_info?: Array<{full_name?: string}>;
		};
		const name = prefs?.profile?.name;
		if (name && name !== dirName) return name;
		const fullName = prefs?.account_info?.[0]?.full_name;
		if (fullName) return fullName;
	} catch {
		// Ignore — fall back to dirName
	}

	return dirName;
}

export function findBrowserProfiles(
	browser: BrowserDescriptor,
): BrowserProfile[] {
	const baseDir = getProfileBaseDir(browser);
	if (!baseDir || !fs.existsSync(baseDir)) return [];

	return fs
		.readdirSync(baseDir, {withFileTypes: true})
		.filter(e => e.isDirectory())
		.map(e => e.name)
		.filter(name => fs.existsSync(path.join(baseDir, name, 'Cookies')))
		.map(dirName => ({
			dirName,
			displayName: getProfileDisplayName(baseDir, dirName),
			browser: browser.id,
		}));
}

export function detectInstalledBrowsers(): BrowserDescriptor[] {
	return Object.values(SUPPORTED_BROWSERS).filter(browser => {
		return findBrowserProfiles(browser).length > 0;
	});
}

export function isBrowserRunning(
	browser: BrowserDescriptor,
	platform: NodeJS.Platform = process.platform,
): boolean {
	const names = browser.processNames[platform as OsPlatform];
	if (!names || names.length === 0) return false;

	try {
		if (platform === 'win32') {
			for (const name of names) {
				try {
					const output = execSync(`tasklist /FI "IMAGENAME eq ${name}" /NH`, {
						encoding: 'utf-8',
						stdio: ['pipe', 'pipe', 'ignore'],
					});
					if (output.includes(name)) return true;
				} catch {
					continue;
				}
			}
			return false;
		}

		// macOS and Linux: pgrep
		for (const name of names) {
			try {
				execSync(`pgrep -x "${name}"`, {stdio: 'ignore'});
				return true;
			} catch {
				continue;
			}
		}
		return false;
	} catch {
		return false;
	}
}

export function collectFiles(
	dirPath: string,
	baseDir: string,
	files: Record<string, Uint8Array> = {},
): Record<string, Uint8Array> {
	if (!fs.existsSync(dirPath)) return files;

	for (const entry of fs.readdirSync(dirPath, {withFileTypes: true})) {
		if (SKIP_NAMES.has(entry.name)) continue;
		if (SKIP_EXTS.has(path.extname(entry.name))) continue;

		const fullPath = path.join(dirPath, entry.name);
		const relPath = path.relative(baseDir, fullPath);

		if (entry.isDirectory()) {
			collectFiles(fullPath, baseDir, files);
		} else if (entry.isFile()) {
			files[relPath] = new Uint8Array(fs.readFileSync(fullPath));
		}
	}

	return files;
}

// ---------------------------------------------------------------------------
// Backward-compatible wrappers
// ---------------------------------------------------------------------------

/**
 * @deprecated Use `findBrowserProfiles(getBrowserDescriptor('chrome'))` instead.
 */
export function findChromeProfiles(): BrowserProfile[] {
	return findBrowserProfiles(SUPPORTED_BROWSERS.chrome);
}

/**
 * @deprecated Use `isBrowserRunning(getBrowserDescriptor('chrome'))` instead.
 */
export function isChromeRunning(): boolean {
	return isBrowserRunning(SUPPORTED_BROWSERS.chrome);
}
