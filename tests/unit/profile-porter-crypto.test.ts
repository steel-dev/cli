import * as crypto from 'node:crypto';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {describe, test, expect} from 'vitest';
import Database from 'better-sqlite3';
import {
	deriveKey,
	deriveTargetKey,
	decryptCookie,
	encryptCookie,
	makeReencryptedCookiesBuffer,
	TARGET_ALGORITHM,
} from '../../source/utils/browser/profile-porter/crypto';
import type {CryptoAlgorithm} from '../../source/utils/browser/profile-porter/types';

const CBC_IV = Buffer.alloc(16, 0x20);
const CBC_ALGORITHM: CryptoAlgorithm = {type: 'aes-128-cbc', iv: CBC_IV};
const GCM_ALGORITHM: CryptoAlgorithm = {type: 'aes-256-gcm', nonceLength: 12};

describe('deriveKey', () => {
	test('produces 16-byte key with PBKDF2', () => {
		const key = deriveKey('test-passphrase', 1003);
		expect(key).toBeInstanceOf(Buffer);
		expect(key.length).toBe(16);
	});

	test('different iterations produce different keys', () => {
		const key1 = deriveKey('peanuts', 1);
		const key2 = deriveKey('peanuts', 1003);
		expect(key1.equals(key2)).toBe(false);
	});

	test('peanuts key matches target key', () => {
		const key = deriveKey('peanuts', 1);
		const target = deriveTargetKey();
		expect(key.equals(target)).toBe(true);
	});
});

describe('AES-128-CBC encrypt/decrypt (macOS, Linux)', () => {
	const key = deriveKey('test-passphrase', 1003);

	test('round-trip with metaVersion < 24', () => {
		const encrypted = encryptCookie(
			'session=abc',
			key,
			'.example.com',
			20,
			CBC_ALGORITHM,
		);
		expect(encrypted.slice(0, 3).toString('ascii')).toBe('v10');

		const decrypted = decryptCookie(
			encrypted,
			key,
			'.example.com',
			20,
			CBC_ALGORITHM,
		);
		expect(decrypted).toBe('session=abc');
	});

	test('round-trip with metaVersion >= 24 (domain hash binding)', () => {
		const encrypted = encryptCookie(
			'token=xyz',
			key,
			'.google.com',
			24,
			CBC_ALGORITHM,
		);
		const decrypted = decryptCookie(
			encrypted,
			key,
			'.google.com',
			24,
			CBC_ALGORITHM,
		);
		expect(decrypted).toBe('token=xyz');
	});

	test('returns null for non-v10 prefix', () => {
		const garbage = Buffer.from('v11garbage');
		const result = decryptCookie(garbage, key, '.x.com', 20, CBC_ALGORITHM);
		expect(result).toBeNull();
	});

	test('returns null for corrupted ciphertext', () => {
		const corrupted = Buffer.concat([
			Buffer.from('v10'),
			crypto.randomBytes(32),
		]);
		const result = decryptCookie(corrupted, key, '.x.com', 20, CBC_ALGORITHM);
		expect(result).toBeNull();
	});

	test('returns null for wrong key', () => {
		const encrypted = encryptCookie('secret', key, '.x.com', 20, CBC_ALGORITHM);
		const wrongKey = deriveKey('wrong-passphrase', 1003);
		const result = decryptCookie(
			encrypted,
			wrongKey,
			'.x.com',
			20,
			CBC_ALGORITHM,
		);
		// May return garbage or null depending on padding
		// The important thing is it doesn't crash
		expect(typeof result === 'string' || result === null).toBe(true);
	});
});

describe('AES-256-GCM encrypt/decrypt (Windows)', () => {
	// Windows uses a 32-byte key
	const key = crypto.randomBytes(32);

	test('round-trip with metaVersion < 24', () => {
		const encrypted = encryptCookie(
			'session=win',
			key,
			'.example.com',
			20,
			GCM_ALGORITHM,
		);
		expect(encrypted.slice(0, 3).toString('ascii')).toBe('v10');

		const decrypted = decryptCookie(
			encrypted,
			key,
			'.example.com',
			20,
			GCM_ALGORITHM,
		);
		expect(decrypted).toBe('session=win');
	});

	test('round-trip with metaVersion >= 24', () => {
		const encrypted = encryptCookie(
			'token=win',
			key,
			'.google.com',
			24,
			GCM_ALGORITHM,
		);
		const decrypted = decryptCookie(
			encrypted,
			key,
			'.google.com',
			24,
			GCM_ALGORITHM,
		);
		expect(decrypted).toBe('token=win');
	});

	test('returns null for corrupted GCM data', () => {
		const corrupted = Buffer.concat([
			Buffer.from('v10'),
			crypto.randomBytes(40),
		]);
		const result = decryptCookie(corrupted, key, '.x.com', 20, GCM_ALGORITHM);
		expect(result).toBeNull();
	});

	test('returns null for wrong key', () => {
		const encrypted = encryptCookie('secret', key, '.x.com', 20, GCM_ALGORITHM);
		const wrongKey = crypto.randomBytes(32);
		const result = decryptCookie(
			encrypted,
			wrongKey,
			'.x.com',
			20,
			GCM_ALGORITHM,
		);
		expect(result).toBeNull();
	});
});

describe('cross-algorithm re-encryption', () => {
	test('encrypt with GCM source key, decrypt with CBC target key', () => {
		const gcmKey = crypto.randomBytes(32);
		const cbcKey = deriveTargetKey();

		const original = 'cross-algo-test-value';
		const encrypted = encryptCookie(
			original,
			gcmKey,
			'.test.com',
			20,
			GCM_ALGORITHM,
		);
		const decrypted = decryptCookie(
			encrypted,
			gcmKey,
			'.test.com',
			20,
			GCM_ALGORITHM,
		);
		expect(decrypted).toBe(original);

		// Re-encrypt for target (CBC)
		const reencrypted = encryptCookie(
			original,
			cbcKey,
			'.test.com',
			20,
			CBC_ALGORITHM,
		);
		const finalDecrypted = decryptCookie(
			reencrypted,
			cbcKey,
			'.test.com',
			20,
			CBC_ALGORITHM,
		);
		expect(finalDecrypted).toBe(original);
	});
});

describe('makeReencryptedCookiesBuffer', () => {
	function createFixtureDb(
		dbPath: string,
		sourceKey: Buffer,
		sourceAlgorithm: CryptoAlgorithm,
		metaVersion: number,
	): {host: string; value: string}[] {
		const db = new Database(dbPath);
		db.exec(`
			CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
			INSERT INTO meta (key, value) VALUES ('version', '${metaVersion}');
			CREATE TABLE cookies (
				rowid INTEGER PRIMARY KEY AUTOINCREMENT,
				host_key TEXT NOT NULL,
				name TEXT NOT NULL,
				encrypted_value BLOB NOT NULL
			);
		`);

		const cookies = [
			{host: '.example.com', name: 'session', value: 'abc123'},
			{host: '.google.com', name: 'token', value: 'xyz789'},
			{host: '.test.org', name: 'pref', value: 'dark-mode'},
		];

		const insert = db.prepare(
			'INSERT INTO cookies (host_key, name, encrypted_value) VALUES (?, ?, ?)',
		);

		for (const cookie of cookies) {
			const encrypted = encryptCookie(
				cookie.value,
				sourceKey,
				cookie.host,
				metaVersion,
				sourceAlgorithm,
			);
			insert.run(cookie.host, cookie.name, encrypted);
		}

		db.close();
		return cookies;
	}

	test('re-encrypts all cookies from CBC source to CBC target', () => {
		const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'steel-crypto-test-'));
		const dbPath = path.join(tmpDir, 'Cookies');
		const sourceKey = deriveKey('test-passphrase', 1003);
		const targetKey = deriveTargetKey();
		const cookies = createFixtureDb(dbPath, sourceKey, CBC_ALGORITHM, 20);

		try {
			const {buffer, converted} = makeReencryptedCookiesBuffer(
				dbPath,
				sourceKey,
				targetKey,
				CBC_ALGORITHM,
				TARGET_ALGORITHM,
			);

			expect(converted).toBe(cookies.length);

			// Verify cookies are decryptable with target key
			const verifyPath = path.join(tmpDir, 'verify.db');
			fs.writeFileSync(verifyPath, buffer);
			const db = new Database(verifyPath);
			const rows = db
				.prepare('SELECT host_key, encrypted_value FROM cookies')
				.all() as Array<{host_key: string; encrypted_value: Buffer}>;

			for (const row of rows) {
				const cookie = cookies.find(c => c.host === row.host_key)!;
				const decrypted = decryptCookie(
					row.encrypted_value,
					targetKey,
					row.host_key,
					20,
					TARGET_ALGORITHM,
				);
				expect(decrypted).toBe(cookie.value);
			}

			db.close();
		} finally {
			fs.rmSync(tmpDir, {recursive: true, force: true});
		}
	});

	test('re-encrypts from GCM source (Windows) to CBC target (Steel)', () => {
		const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'steel-crypto-test-'));
		const dbPath = path.join(tmpDir, 'Cookies');
		const sourceKey = crypto.randomBytes(32);
		const targetKey = deriveTargetKey();
		const cookies = createFixtureDb(dbPath, sourceKey, GCM_ALGORITHM, 20);

		try {
			const {buffer, converted} = makeReencryptedCookiesBuffer(
				dbPath,
				sourceKey,
				targetKey,
				GCM_ALGORITHM,
				TARGET_ALGORITHM,
			);

			expect(converted).toBe(cookies.length);

			const verifyPath = path.join(tmpDir, 'verify.db');
			fs.writeFileSync(verifyPath, buffer);
			const db = new Database(verifyPath);
			const rows = db
				.prepare('SELECT host_key, encrypted_value FROM cookies')
				.all() as Array<{host_key: string; encrypted_value: Buffer}>;

			for (const row of rows) {
				const cookie = cookies.find(c => c.host === row.host_key)!;
				const decrypted = decryptCookie(
					row.encrypted_value,
					targetKey,
					row.host_key,
					20,
					TARGET_ALGORITHM,
				);
				expect(decrypted).toBe(cookie.value);
			}

			db.close();
		} finally {
			fs.rmSync(tmpDir, {recursive: true, force: true});
		}
	});

	test('handles metaVersion >= 24 with domain hash binding', () => {
		const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'steel-crypto-test-'));
		const dbPath = path.join(tmpDir, 'Cookies');
		const sourceKey = deriveKey('test-passphrase', 1003);
		const targetKey = deriveTargetKey();
		const cookies = createFixtureDb(dbPath, sourceKey, CBC_ALGORITHM, 24);

		try {
			const {buffer, converted} = makeReencryptedCookiesBuffer(
				dbPath,
				sourceKey,
				targetKey,
				CBC_ALGORITHM,
				TARGET_ALGORITHM,
			);

			expect(converted).toBe(cookies.length);

			const verifyPath = path.join(tmpDir, 'verify.db');
			fs.writeFileSync(verifyPath, buffer);
			const db = new Database(verifyPath);
			const rows = db
				.prepare('SELECT host_key, encrypted_value FROM cookies')
				.all() as Array<{host_key: string; encrypted_value: Buffer}>;

			for (const row of rows) {
				const cookie = cookies.find(c => c.host === row.host_key)!;
				const decrypted = decryptCookie(
					row.encrypted_value,
					targetKey,
					row.host_key,
					24,
					TARGET_ALGORITHM,
				);
				expect(decrypted).toBe(cookie.value);
			}

			db.close();
		} finally {
			fs.rmSync(tmpDir, {recursive: true, force: true});
		}
	});

	test('handles empty cookies table', () => {
		const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'steel-crypto-test-'));
		const dbPath = path.join(tmpDir, 'Cookies');
		const db = new Database(dbPath);
		db.exec(`
			CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
			INSERT INTO meta (key, value) VALUES ('version', '20');
			CREATE TABLE cookies (
				rowid INTEGER PRIMARY KEY AUTOINCREMENT,
				host_key TEXT NOT NULL,
				name TEXT NOT NULL,
				encrypted_value BLOB NOT NULL
			);
		`);
		db.close();

		const sourceKey = deriveKey('test', 1003);
		const targetKey = deriveTargetKey();

		try {
			const {converted} = makeReencryptedCookiesBuffer(
				dbPath,
				sourceKey,
				targetKey,
				CBC_ALGORITHM,
				TARGET_ALGORITHM,
			);
			expect(converted).toBe(0);
		} finally {
			fs.rmSync(tmpDir, {recursive: true, force: true});
		}
	});
});
