import * as crypto from 'node:crypto';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import Database from 'better-sqlite3';
import type {CryptoAlgorithm} from './types.js';

const CBC_IV = Buffer.alloc(16, 0x20); // 16 space bytes

export const TARGET_ALGORITHM: CryptoAlgorithm = {
	type: 'aes-128-cbc',
	iv: CBC_IV,
};

export function deriveKey(passphrase: string, iterations: number): Buffer {
	return crypto.pbkdf2Sync(passphrase, 'saltysalt', iterations, 16, 'sha1');
}

export function deriveTargetKey(): Buffer {
	return deriveKey('peanuts', 1);
}

export function decryptCookie(
	encryptedValue: Buffer,
	key: Buffer,
	hostKey: string,
	metaVersion: number,
	algorithm: CryptoAlgorithm,
): string | null {
	const prefix = encryptedValue.slice(0, 3).toString('ascii');
	if (prefix !== 'v10' && prefix !== 'v20') return null;

	if (algorithm.type === 'aes-128-cbc') {
		const decipher = crypto.createDecipheriv('aes-128-cbc', key, algorithm.iv);
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

	if (algorithm.type === 'aes-256-gcm') {
		// v10/v20 prefix (3 bytes) + nonce (12 bytes) + ciphertext + auth tag (16 bytes)
		const payload = encryptedValue.slice(3);
		if (payload.length < algorithm.nonceLength + 16) return null;

		const nonce = payload.slice(0, algorithm.nonceLength);
		const tag = payload.slice(-16);
		const ciphertext = payload.slice(algorithm.nonceLength, -16);

		try {
			const decipher = crypto.createDecipheriv('aes-256-gcm', key, nonce);
			decipher.setAuthTag(tag);
			const plaintext = Buffer.concat([
				decipher.update(ciphertext),
				decipher.final(),
			]);

			if (metaVersion >= 24 && plaintext.length >= 32) {
				const expectedHash = crypto
					.createHash('sha256')
					.update(hostKey)
					.digest();
				if (expectedHash.equals(plaintext.slice(0, 32))) {
					return plaintext.slice(32).toString('utf-8');
				}
			}

			return plaintext.toString('utf-8');
		} catch {
			return null;
		}
	}

	return null;
}

export function encryptCookie(
	value: string,
	key: Buffer,
	hostKey: string,
	metaVersion: number,
	algorithm: CryptoAlgorithm,
): Buffer {
	let plaintext = Buffer.from(value, 'utf-8');

	if (metaVersion >= 24) {
		const domainHash = crypto.createHash('sha256').update(hostKey).digest();
		plaintext = Buffer.concat([domainHash, plaintext]);
	}

	if (algorithm.type === 'aes-128-cbc') {
		const cipher = crypto.createCipheriv('aes-128-cbc', key, algorithm.iv);
		cipher.setAutoPadding(true);
		const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final()]);
		return Buffer.concat([Buffer.from('v10'), encrypted]);
	}

	if (algorithm.type === 'aes-256-gcm') {
		const nonce = crypto.randomBytes(algorithm.nonceLength);
		const cipher = crypto.createCipheriv('aes-256-gcm', key, nonce);
		const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final()]);
		const tag = cipher.getAuthTag();
		return Buffer.concat([Buffer.from('v10'), nonce, encrypted, tag]);
	}

	throw new Error(
		`Unsupported algorithm: ${(algorithm as CryptoAlgorithm).type}`,
	);
}

export function makeReencryptedCookiesBuffer(
	originalPath: string,
	sourceKey: Buffer,
	targetKey: Buffer,
	sourceAlgorithm: CryptoAlgorithm,
	targetAlgorithm: CryptoAlgorithm,
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
						sourceKey,
						row.host_key,
						metaVersion,
						sourceAlgorithm,
					);
					if (plaintext === null) continue;

					const reencrypted = encryptCookie(
						plaintext,
						targetKey,
						row.host_key,
						metaVersion,
						targetAlgorithm,
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
