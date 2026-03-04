import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {describe, test, expect} from 'vitest';
import {
	readSteelProfile,
	writeSteelProfile,
} from '../../source/utils/browser/lifecycle/profile-store';

function createTempDir(): string {
	return fs.mkdtempSync(path.join(os.tmpdir(), 'steel-profile-store-test-'));
}

describe('profile-store', () => {
	describe('readSteelProfile', () => {
		test('returns null when .steel.json does not exist', async () => {
			const dir = createTempDir();

			try {
				const result = await readSteelProfile(dir);
				expect(result).toBeNull();
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('returns profileId from existing .steel.json', async () => {
			const dir = createTempDir();

			try {
				fs.writeFileSync(
					path.join(dir, '.steel.json'),
					JSON.stringify({
						profileId: 'uuid-abc-123',
						updatedAt: '2026-03-04T00:00:00.000Z',
					}),
					'utf-8',
				);

				const result = await readSteelProfile(dir);
				expect(result).toEqual({profileId: 'uuid-abc-123'});
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('returns null when .steel.json contains invalid JSON', async () => {
			const dir = createTempDir();

			try {
				fs.writeFileSync(path.join(dir, '.steel.json'), 'not-json', 'utf-8');

				const result = await readSteelProfile(dir);
				expect(result).toBeNull();
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('returns null when .steel.json has no profileId field', async () => {
			const dir = createTempDir();

			try {
				fs.writeFileSync(
					path.join(dir, '.steel.json'),
					JSON.stringify({updatedAt: '2026-03-04T00:00:00.000Z'}),
					'utf-8',
				);

				const result = await readSteelProfile(dir);
				expect(result).toBeNull();
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('expands ~ to home directory', async () => {
			const homeDir = os.homedir();
			const subDirName = `steel-profile-tilde-test-${Date.now()}`;
			const fullDir = path.join(homeDir, subDirName);

			try {
				fs.mkdirSync(fullDir, {recursive: true});
				fs.writeFileSync(
					path.join(fullDir, '.steel.json'),
					JSON.stringify({profileId: 'tilde-profile-id'}),
					'utf-8',
				);

				const result = await readSteelProfile(`~/${subDirName}`);
				expect(result).toEqual({profileId: 'tilde-profile-id'});
			} finally {
				fs.rmSync(fullDir, {recursive: true, force: true});
			}
		});
	});

	describe('writeSteelProfile', () => {
		test('creates .steel.json with profileId and updatedAt', async () => {
			const dir = createTempDir();

			try {
				await writeSteelProfile(dir, 'new-profile-uuid');

				const filePath = path.join(dir, '.steel.json');
				expect(fs.existsSync(filePath)).toBe(true);

				const contents = JSON.parse(fs.readFileSync(filePath, 'utf-8')) as {
					profileId: string;
					updatedAt: string;
				};
				expect(contents.profileId).toBe('new-profile-uuid');
				expect(typeof contents.updatedAt).toBe('string');
				expect(() => new Date(contents.updatedAt)).not.toThrow();
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('creates directory recursively if it does not exist', async () => {
			const baseDir = createTempDir();
			const nestedDir = path.join(baseDir, 'deep', 'nested', 'dir');

			try {
				await writeSteelProfile(nestedDir, 'nested-profile-uuid');

				expect(fs.existsSync(path.join(nestedDir, '.steel.json'))).toBe(true);
			} finally {
				fs.rmSync(baseDir, {recursive: true, force: true});
			}
		});

		test('overwrites existing .steel.json', async () => {
			const dir = createTempDir();

			try {
				await writeSteelProfile(dir, 'first-profile-uuid');
				await writeSteelProfile(dir, 'second-profile-uuid');

				const contents = JSON.parse(
					fs.readFileSync(path.join(dir, '.steel.json'), 'utf-8'),
				) as {profileId: string};
				expect(contents.profileId).toBe('second-profile-uuid');
			} finally {
				fs.rmSync(dir, {recursive: true, force: true});
			}
		});

		test('expands ~ to home directory when writing', async () => {
			const homeDir = os.homedir();
			const subDirName = `steel-profile-write-tilde-test-${Date.now()}`;
			const fullDir = path.join(homeDir, subDirName);

			try {
				await writeSteelProfile(`~/${subDirName}`, 'tilde-write-uuid');

				const filePath = path.join(fullDir, '.steel.json');
				expect(fs.existsSync(filePath)).toBe(true);

				const contents = JSON.parse(fs.readFileSync(filePath, 'utf-8')) as {
					profileId: string;
				};
				expect(contents.profileId).toBe('tilde-write-uuid');
			} finally {
				fs.rmSync(fullDir, {recursive: true, force: true});
			}
		});
	});
});
