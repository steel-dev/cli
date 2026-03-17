// Types
export type {
	BrowserId,
	BrowserDescriptor,
	BrowserProfile,
	ChromeProfile,
	CryptoAlgorithm,
	KeyProvider,
	PackageResult,
	PackageProfileOptions,
	SyncProfileOptions,
	SyncProfileResult,
} from './types.js';

// Browser registry
export {
	SUPPORTED_BROWSERS,
	getBrowserDescriptor,
	getProfileBaseDir,
} from './browsers.js';

// Discovery
export {
	findBrowserProfiles,
	detectInstalledBrowsers,
	isBrowserRunning,
	getProfileDisplayName,
	collectFiles,
	INCLUDE_ENTRIES,
	// Backward-compatible wrappers
	findChromeProfiles,
	isChromeRunning,
} from './discovery.js';

// Crypto
export {
	deriveKey,
	deriveTargetKey,
	decryptCookie,
	encryptCookie,
	makeReencryptedCookiesBuffer,
	TARGET_ALGORITHM,
} from './crypto.js';

// Key providers
export {createKeyProvider} from './key-providers.js';

// Packager
export {packageProfile, packageChromeProfile} from './packager.js';

// API
export {uploadProfileToSteel, updateProfileOnSteel} from './api.js';
