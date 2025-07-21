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
	reactMode?: boolean; // Add this new option
}

interface UpdateCache {
	lastCheck: number;
	lastKnownVersion: string;
	hasUpdate: boolean;
}

// Global update state for React components
export interface UpdateState {
	status: 'idle' | 'checking' | 'updating' | 'complete' | 'error';
	message: string;
	error?: string;
	versionInfo?: VersionInfo;
}

let globalUpdateState: UpdateState = {
	status: 'idle',
	message: '',
};

let updateStateListeners: ((state: UpdateState) => void)[] = [];

/**
 * Subscribe to update state changes
 */
export function subscribeToUpdateState(
	callback: (state: UpdateState) => void,
): () => void {
	updateStateListeners.push(callback);
	// Return unsubscribe function
	return () => {
		updateStateListeners = updateStateListeners.filter(cb => cb !== callback);
	};
}

/**
 * Update the global update state and notify listeners
 */
function setUpdateState(newState: Partial<UpdateState>): void {
	globalUpdateState = {...globalUpdateState, ...newState};
	updateStateListeners.forEach(callback => callback(globalUpdateState));
}

/**
 * Get current update state
 */
export function getUpdateState(): UpdateState {
	return globalUpdateState;
}

// Global variable to store update information for React components
let globalUpdateInfo: VersionInfo | null = null;

/**
 * Set global update info for React components to access
 */
export function setGlobalUpdateInfo(versionInfo: VersionInfo | null): void {
	globalUpdateInfo = versionInfo;
}

/**
 * Get global update info for React components
 */
export function getGlobalUpdateInfo(): VersionInfo | null {
	return globalUpdateInfo;
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
	} catch {
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
	} catch {
		// Do nothing
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
 * Returns true if latest > current
 */
export function compareVersions(current: string, latest: string): boolean {
	// Parse version parts including pre-release identifiers
	const parseVersion = (version: string) => {
		const parts = version.split('-');
		const mainVersion = parts[0];
		const preRelease = parts.slice(1).join('-');

		const [major, minor, patch] = mainVersion.split('.').map(Number);

		return {
			major: major || 0,
			minor: minor || 0,
			patch: patch || 0,
			preRelease,
		};
	};

	const currentParsed = parseVersion(current);
	const latestParsed = parseVersion(latest);

	// Compare major.minor.patch first
	if (latestParsed.major > currentParsed.major) return true;
	if (latestParsed.major < currentParsed.major) return false;

	if (latestParsed.minor > currentParsed.minor) return true;
	if (latestParsed.minor < currentParsed.minor) return false;

	if (latestParsed.patch > currentParsed.patch) return true;
	if (latestParsed.patch < currentParsed.patch) return false;

	// If major.minor.patch are equal, compare pre-release versions
	// No pre-release (release version) > pre-release version
	if (!latestParsed.preRelease && currentParsed.preRelease) return true;
	if (latestParsed.preRelease && !currentParsed.preRelease) return false;

	// Both have pre-release versions, compare them
	if (latestParsed.preRelease && currentParsed.preRelease) {
		// For pre-release comparison, we'll do string comparison
		// This handles cases like beta.1 vs beta.2, alpha vs beta, etc.
		return latestParsed.preRelease > currentParsed.preRelease;
	}

	// Versions are equal
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

	const versionInfo = {
		current,
		latest,
		hasUpdate,
	};

	setUpdateState({
		versionInfo,
		message: hasUpdate
			? `Update available: v${current} → v${latest}`
			: 'Already up to date',
	});

	return versionInfo;
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
		setUpdateState({
			status: 'updating',
			message: 'Updating Steel CLI...',
		});

		// Only log to console if not in React mode and not silent
		if (!options.silent && !options.reactMode) {
			console.log('🔄 Updating Steel CLI...');
		}

		const updateProcess = spawn(
			'npm',
			['install', '-g', `${packageName}@latest`],
			{
				stdio: options.silent || options.reactMode ? 'pipe' : 'inherit',
				shell: true,
			},
		);

		updateProcess.on('close', code => {
			if (code === 0) {
				setUpdateState({
					status: 'complete',
					message: 'Steel CLI updated successfully!',
				});

				if (!options.silent && !options.reactMode) {
					console.log('✅ Steel CLI updated successfully!');
					console.log('🔄 Please restart your command to use the new version.');
				}
				resolve(true);
			} else {
				setUpdateState({
					status: 'error',
					message: 'Failed to update Steel CLI',
					error:
						'Update process failed. You may need to run with sudo or check your npm permissions.',
				});

				if (!options.silent && !options.reactMode) {
					console.error('❌ Failed to update Steel CLI');
					console.error(
						'You may need to run with sudo or check your npm permissions',
					);
				}
				resolve(false);
			}
		});

		updateProcess.on('error', error => {
			setUpdateState({
				status: 'error',
				message: 'Error during update',
				error: error.message,
			});

			if (!options.silent && !options.reactMode) {
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
	try {
		// Check cache first (unless forced)
		if (!options.force && shouldSkipUpdateCheck()) {
			const cache = readUpdateCache();
			if (cache) {
				const versionInfo = {
					current: getCurrentVersion(),
					latest: cache.lastKnownVersion,
					hasUpdate: cache.hasUpdate,
				};

				setUpdateState({
					status: 'complete',
					message: cache.hasUpdate
						? 'Update available (cached)'
						: 'Already up to date (cached)',
					versionInfo,
				});

				return versionInfo;
			}
		}

		const versionInfo = await checkForUpdates();

		if (versionInfo.hasUpdate) {
			// Get changelog if available
			const changelog = await getChangelog();
			if (changelog) {
				versionInfo.changelog = changelog;
			}

			if (options.autoUpdate || options.force) {
				const success = await performUpdate('@steel-dev/cli', options);

				if (success) {
					const updatedCache: UpdateCache = {
						lastCheck: Date.now(),
						lastKnownVersion: versionInfo.latest,
						hasUpdate: false,
					};
					writeUpdateCache(updatedCache);

					versionInfo.current = versionInfo.latest;
					versionInfo.hasUpdate = false;

					setUpdateState({
						status: 'complete',
						message: 'Please restart your command to use the new version.',
						versionInfo,
					});
				}

				// In React mode, don't exit the process - let the React component handle the UI
				if (success && !options.silent && !options.reactMode) {
					console.log('🚀 Restart your command to use the new version!\n');
					process.exit(0);
				}
			}
		} else {
			setUpdateState({
				status: 'complete',
				message: 'Already up to date',
				versionInfo,
			});
		}

		return versionInfo;
	} catch (error) {
		setUpdateState({
			status: 'error',
			message: 'Update check failed',
			error: error instanceof Error ? error.message : 'Unknown error',
		});
		throw error;
	}
}
