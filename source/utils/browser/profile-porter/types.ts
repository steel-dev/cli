export type BrowserId =
	| 'chrome'
	| 'edge'
	| 'brave'
	| 'arc'
	| 'opera'
	| 'vivaldi';

export type OsPlatform = 'darwin' | 'win32' | 'linux';

export type BrowserDescriptor = {
	id: BrowserId;
	displayName: string;
	/** Profile base directory relative to the OS-specific application data root. */
	profileBaseDirs: Partial<Record<OsPlatform, string>>;
	/** Process names used by `isBrowserRunning`. */
	processNames: Partial<Record<OsPlatform, string[]>>;
	/** Keychain / keyring service identifier per OS. */
	keychainService: Partial<Record<OsPlatform, string>>;
};

export type BrowserProfile = {
	dirName: string;
	displayName: string;
	browser: BrowserId;
};

/**
 * Backward-compatible alias. Prefer `BrowserProfile` in new code.
 */
export type ChromeProfile = {
	dirName: string;
	displayName: string;
};

export type CryptoAlgorithm =
	| {type: 'aes-128-cbc'; iv: Buffer}
	| {type: 'aes-256-gcm'; nonceLength: number};

export type KeyProvider = {
	getKey(): Promise<Buffer>;
	getTargetKey(): Buffer;
	sourceAlgorithm: CryptoAlgorithm;
	targetAlgorithm: CryptoAlgorithm;
};

export type PackageResult = {
	zipBuffer: Buffer;
	cookiesReencrypted: number;
};

export type PackageProfileOptions = {
	profileDir: string;
	keyProvider: KeyProvider;
	onProgress?: (msg: string) => void;
	onKeyPrompt?: () => void;
};

export type SyncProfileOptions = {
	name: string;
	chromeProfile?: string;
};

export type SyncProfileResult = {
	profileId: string;
	cookiesReencrypted: number;
	zipBytes: number;
};
