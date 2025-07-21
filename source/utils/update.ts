import {spawn} from 'child_process';
import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import {CONFIG_DIR} from './constants.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const UPDATE_CACHE_FILE = path.join(CONFIG_DIR, 'update-cache.json');
const UPDATE_CHECK_INTERVAL = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

export interface VersionInfo {
	current: string;
	latest: string;
	hasUpdate: boolean;
	changelog?: string;
}

export interface UpdateOptions {
	force?: boolean;
	silent?: boolean;
}

interface UpdateCache {
	lastCheck: number;
	lastKnownVersion: string;
	hasUpdate: boolean;
}

/**
 * Get the current version from package.json
 */
export function getCurrentVersion(): string {
	try {
		const packagePath = path.resolve(__dirname, '../../package.json');
		const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
		return packageJson.version;
	} catch (error) {
		console.warn('Could not read current version:', error);
		return '0.0.0';
	}
}

/**
 * Read update cache from disk
 */
function readUpdateCache(): UpdateCache | null {
	try {
		if (!fs.existsSync(UPDATE_CACHE_FILE)) {
			return null;
		}
		const cacheData = fs.readFileSync(UPDATE_CACHE_FILE, 'utf8');
		return JSON.parse(cacheData);
	} catch (error) {
		console.debug('Could not read update cache:', error);
		return null;
	}
}

/**
 * Write update cache to disk
 */
function writeUpdateCache(cache: UpdateCache): void {
	try {
		// Ensure config directory exists
		if (!fs.existsSync(CONFIG_DIR)) {
			fs.mkdirSync(CONFIG_DIR, {recursive: true});
		}
		fs.writeFileSync(UPDATE_CACHE_FILE, JSON.stringify(cache, null, 2));
	} catch (error) {
		console.debug('Could not write update cache:', error);
	}
}

/**
 * Check if we should skip the update check based on cache
 */
function shouldSkipUpdateCheck(): boolean {
	const cache = readUpdateCache();
	if (!cache) return false;

	const now = Date.now();
	const timeSinceLastCheck = now - cache.lastCheck;

	// Skip if we checked recently and no update was available
	return timeSinceLastCheck < UPDATE_CHECK_INTERVAL && !cache.hasUpdate;
}

/**
 * Check for the latest version available on npm
 */
export async function getLatestVersion(
	packageName = '@steel-dev/cli',
): Promise<string> {
	try {
		const response = await fetch(
			`https://registry.npmjs.org/${packageName}/latest`,
		);
		if (!response.ok) {
			throw new Error(`Failed to fetch version info: ${response.statusText}`);
		}
		const data = await response.json();
		return data.version;
	} catch (error) {
		console.warn('Could not check for updates:', error);
		return getCurrentVersion();
	}
}

/**
 * Compare two semantic version strings
 */
export function compareVersions(current: string, latest: string): boolean {
	const currentParts = current
		.replace(/[^\d.]/g, '')
		.split('.')
		.map(Number);
	const latestParts = latest
		.replace(/[^\d.]/g, '')
		.split('.')
		.map(Number);

	// Pad arrays to same length
	const maxLength = Math.max(currentParts.length, latestParts.length);
	while (currentParts.length < maxLength) currentParts.push(0);
	while (latestParts.length < maxLength) latestParts.push(0);

	for (let i = 0; i < maxLength; i++) {
		if (latestParts[i] > currentParts[i]) return true;
		if (latestParts[i] < currentParts[i]) return false;
	}

	return false;
}

/**
 * Check if an update is available
 */
export async function checkForUpdates(
	packageName = '@steel-dev/cli',
): Promise<VersionInfo> {
	const current = getCurrentVersion();
	const latest = await getLatestVersion(packageName);
	const hasUpdate = compareVersions(current, latest);

	// Update cache
	const cache: UpdateCache = {
		lastCheck: Date.now(),
		lastKnownVersion: latest,
		hasUpdate,
	};
	writeUpdateCache(cache);

	return {
		current,
		latest,
		hasUpdate,
	};
}

/**
 * Get changelog information for the latest version
 */
export async function getChangelog(
	packageName = '@steel-dev/cli',
): Promise<string | undefined> {
	try {
		const response = await fetch(`https://registry.npmjs.org/${packageName}`);
		if (!response.ok) return undefined;

		const data = await response.json();
		const latestVersion = data['dist-tags'].latest;
		const versionData = data.versions[latestVersion];

		// Try to get changelog from various sources
		if (versionData.changelog) return versionData.changelog;
		if (versionData.description) return versionData.description;

		return undefined;
	} catch (error) {
		console.warn('Could not fetch changelog:', error);
		return undefined;
	}
}

/**
 * Perform the actual update using npm
 */
export async function performUpdate(
	packageName = '@steel-dev/cli',
	options: UpdateOptions = {},
): Promise<boolean> {
	return new Promise(resolve => {
		if (!options.silent) {
			console.log('🔄 Updating Steel CLI...');
		}

		const updateProcess = spawn(
			'npm',
			['install', '-g', `${packageName}@latest`],
			{
				stdio: options.silent ? 'pipe' : 'inherit',
				shell: true,
			},
		);

		updateProcess.on('close', code => {
			if (code === 0) {
				if (!options.silent) {
					console.log('✅ Steel CLI updated successfully!');
					console.log('🔄 Please restart your command to use the new version.');
				}
				resolve(true);
			} else {
				if (!options.silent) {
					console.error('❌ Failed to update Steel CLI');
					console.error(
						'You may need to run with sudo or check your npm permissions',
					);
				}
				resolve(false);
			}
		});

		updateProcess.on('error', error => {
			if (!options.silent) {
				console.error('❌ Error during update:', error.message);
			}
			resolve(false);
		});
	});
}

/**
 * Check for updates and optionally auto-update
 */
export async function checkAndUpdate(
	options: UpdateOptions & {autoUpdate?: boolean} = {},
): Promise<VersionInfo> {
	// Check cache first (unless forced)
	if (!options.force && shouldSkipUpdateCheck()) {
		const cache = readUpdateCache();
		if (cache) {
			return {
				current: getCurrentVersion(),
				latest: cache.lastKnownVersion,
				hasUpdate: cache.hasUpdate,
			};
		}
	}

	const versionInfo = await checkForUpdates();

	if (versionInfo.hasUpdate) {
		if (!options.silent) {
			console.log(`\n📦 Steel CLI update available!`);
			console.log(`   Current: v${versionInfo.current}`);
			console.log(`   Latest:  v${versionInfo.latest}\n`);
		}

		// Get changelog if available
		const changelog = await getChangelog();
		if (changelog) {
			versionInfo.changelog = changelog;
		}

		if (options.autoUpdate || options.force) {
			const success = await performUpdate('@steel-dev/cli', options);
			if (success && !options.silent) {
				console.log('🚀 Restart your command to use the new version!\n');
				process.exit(0);
			}
		} else if (!options.silent) {
			console.log('💡 Run `steel update` to update to the latest version\n');
		}
	}

	return versionInfo;
}
