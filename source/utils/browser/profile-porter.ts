import * as crypto from 'node:crypto';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {execSync} from 'node:child_process';
import {zipSync} from 'fflate';
import Database from 'better-sqlite3';

export type SyncProfileOptions = {
	name: string;
	chromeProfile?: string;
};

export type SyncProfileResult = {
	profileId: string;
	cookiesReencrypted: number;
	zipBytes: number;
};

const CHROME_BASE_DIR = path.join(
	os.homedir(),
	'Library',
	'Application Support',
	'Google',
	'Chrome',
);

const IV = Buffer.alloc(16, 0x20); // 16 space bytes

const INCLUDE_ENTRIES = [
	'Cookies',
	'Local Storage',
	'IndexedDB',
	'Preferences',
	'Bookmarks',
	'Favicons',
	'History',
	'Web Data',
];

const SKIP_NAMES = new Set([
	'LOCK',
	'SingletonLock',
	'SingletonCookie',
	'SingletonSocket',
]);

const SKIP_EXTS = new Set(['.log', '.pma']);

function getKeychainPassphrase(): string {
	return execSync(
		'security find-generic-password -w -s "Chrome Safe Storage"',
		{
			encoding: 'utf-8',
		},
	).trim();
}

function deriveKey(passphrase: string, iterations: number): Buffer {
	return crypto.pbkdf2Sync(passphrase, 'saltysalt', iterations, 16, 'sha1');
}

function decryptCookie(
	encryptedValue: Buffer,
	key: Buffer,
	hostKey: string,
	metaVersion: number,
): string | null {
	const prefix = encryptedValue.slice(0, 3).toString('ascii');
	if (prefix !== 'v10') return null;

	const decipher = crypto.createDecipheriv('aes-128-cbc', key, IV);
	decipher.setAutoPadding(true);

	let plaintext: Buffer;
	try {
		plaintext = Buffer.concat([
			decipher.update(encryptedValue.slice(3)),
			decipher.final(),
		]);
	} catch {
		return null;
	}

	if (metaVersion >= 24 && plaintext.length >= 32) {
		const expectedHash = crypto.createHash('sha256').update(hostKey).digest();
		if (expectedHash.equals(plaintext.slice(0, 32))) {
			return plaintext.slice(32).toString('utf-8');
		}
	}

	return plaintext.toString('utf-8');
}

function encryptCookie(
	value: string,
	key: Buffer,
	hostKey: string,
	metaVersion: number,
): Buffer {
	let plaintext = Buffer.from(value, 'utf-8');

	if (metaVersion >= 24) {
		const domainHash = crypto.createHash('sha256').update(hostKey).digest();
		plaintext = Buffer.concat([domainHash, plaintext]);
	}

	const cipher = crypto.createCipheriv('aes-128-cbc', key, IV);
	cipher.setAutoPadding(true);
	const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final()]);
	return Buffer.concat([Buffer.from('v10'), encrypted]);
}

function makeReencryptedCookiesBuffer(
	originalPath: string,
	macosKey: Buffer,
	peanutsKey: Buffer,
): {buffer: Buffer; converted: number} {
	const tmpPath = path.join(
		os.tmpdir(),
		`steel-cookies-${Date.now()}-${Math.random().toString(36).slice(2)}.db`,
	);
	fs.copyFileSync(originalPath, tmpPath);

	try {
		const db = new Database(tmpPath);
		try {
			const metaVersion = Number(
				(
					db.prepare("SELECT value FROM meta WHERE key='version'").get() as
						| {value: string}
						| undefined
				)?.value ?? 0,
			);

			const rows = db
				.prepare(
					'SELECT rowid, host_key, encrypted_value FROM cookies WHERE length(encrypted_value) > 3',
				)
				.all() as Array<{
				rowid: number;
				host_key: string;
				encrypted_value: Buffer;
			}>;

			const update = db.prepare(
				'UPDATE cookies SET encrypted_value = ? WHERE rowid = ?',
			);

			let converted = 0;

			db.transaction(() => {
				for (const row of rows) {
					const plaintext = decryptCookie(
						row.encrypted_value,
						macosKey,
						row.host_key,
						metaVersion,
					);
					if (plaintext === null) continue;

					const reencrypted = encryptCookie(
						plaintext,
						peanutsKey,
						row.host_key,
						metaVersion,
					);
					update.run(reencrypted, row.rowid);
					converted++;
				}
			})();

			db.close();

			const buffer = fs.readFileSync(tmpPath);
			return {buffer, converted};
		} catch (error) {
			db.close();
			throw error;
		}
	} finally {
		try {
			fs.unlinkSync(tmpPath);
		} catch {
			// best-effort cleanup
		}
	}
}

function collectFiles(
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

export type ChromeProfile = {
	dirName: string;
	displayName: string;
};

export function findChromeProfiles(): ChromeProfile[] {
	if (!fs.existsSync(CHROME_BASE_DIR)) return [];
	return fs
		.readdirSync(CHROME_BASE_DIR, {withFileTypes: true})
		.filter(e => e.isDirectory())
		.map(e => e.name)
		.filter(name => fs.existsSync(path.join(CHROME_BASE_DIR, name, 'Cookies')))
		.map(dirName => ({
			dirName,
			displayName: getChromeProfileDisplayName(dirName),
		}));
}

function getChromeProfileDisplayName(dirName: string): string {
	try {
		const prefsPath = path.join(CHROME_BASE_DIR, dirName, 'Preferences');
		const prefs = JSON.parse(fs.readFileSync(prefsPath, 'utf-8')) as {
			profile?: {name?: string};
			account_info?: Array<{full_name?: string}>;
		};
		const name = prefs?.profile?.name;
		if (name && name !== dirName) return name;
		const fullName = prefs?.account_info?.[0]?.full_name;
		if (fullName) return fullName;
	} catch {
		console.error(`Error getting Chrome profile display name for ${dirName}`);
	}

	return dirName;
}

export function isChromeRunning(): boolean {
	try {
		execSync('pgrep -x "Google Chrome"', {stdio: 'ignore'});
		return true;
	} catch {
		return false;
	}
}

export type PackageResult = {
	zipBuffer: Buffer;
	cookiesReencrypted: number;
};

export function packageChromeProfile(
	chromeProfile: string,
	onProgress?: (msg: string) => void,
): PackageResult {
	const profileDir = path.join(CHROME_BASE_DIR, chromeProfile);

	if (!fs.existsSync(path.join(profileDir, 'Cookies'))) {
		throw new Error(
			`Chrome profile "${chromeProfile}" not found at ${profileDir}`,
		);
	}

	onProgress?.('Reading Keychain passphrase...');
	const passphrase = getKeychainPassphrase();
	const macosKey = deriveKey(passphrase, 1003);
	const peanutsKey = deriveKey('peanuts', 1);

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
					macosKey,
					peanutsKey,
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

export async function updateProfileOnSteel(
	profileId: string,
	zipBuffer: Buffer,
	apiKey: string,
	apiBase: string,
): Promise<void> {
	const form = new FormData();
	form.append(
		'userDataDir',
		new Blob([new Uint8Array(zipBuffer)], {type: 'application/zip'}),
		'userDataDir.zip',
	);

	const res = await fetch(`${apiBase}/profiles/${profileId}`, {
		method: 'PATCH',
		headers: {'Steel-Api-Key': apiKey},
		body: form,
	});

	if (!res.ok) {
		const body = (await res.json()) as {message?: string};
		throw new Error(
			`Profile update failed (${res.status}): ${body.message ?? JSON.stringify(body)}`,
		);
	}
}

export async function uploadProfileToSteel(
	zipBuffer: Buffer,
	apiKey: string,
	apiBase: string,
): Promise<string> {
	const form = new FormData();
	form.append(
		'userDataDir',
		new Blob([new Uint8Array(zipBuffer)], {type: 'application/zip'}),
		'userDataDir.zip',
	);

	const res = await fetch(`${apiBase}/profiles`, {
		method: 'POST',
		headers: {'Steel-Api-Key': apiKey},
		body: form,
	});

	const body = (await res.json()) as {id?: string; message?: string};

	if (!res.ok) {
		throw new Error(
			`Profile upload failed (${res.status}): ${body.message ?? JSON.stringify(body)}`,
		);
	}

	if (!body.id) {
		throw new Error('Profile upload response missing id');
	}

	return body.id;
}
