import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {describe, test, expect} from 'vitest';
import {
	readSteelProfile,
	writeSteelProfile,
	validateProfileName,
	listSteelProfiles,
	deleteSteelProfile,
} from '../../source/utils/browser/lifecycle/profile-store';

function createTempConfigDir(): string {
	return fs.mkdtempSync(path.join(os.tmpdir(), 'steel-profile-store-test-'));
}

describe('validateProfileName', () => {
	test('accepts simple names', () => {
		expect(validateProfileName('myapp')).toBeNull();
		expect(validateProfileName('my-app')).toBeNull();
		expect(validateProfileName('my_app')).toBeNull();
		expect(validateProfileName('MyApp123')).toBeNull();
	});

	test('rejects empty or whitespace-only names', () => {
		expect(validateProfileName('')).not.toBeNull();
		expect(validateProfileName('   ')).not.toBeNull();
	});

	test('rejects names containing path separators', () => {
		expect(validateProfileName('my/app')).not.toBeNull();
		expect(validateProfileName('my\\app')).not.toBeNull();
		expect(validateProfileName('../etc/passwd')).not.toBeNull();
	});
});

describe('readSteelProfile', () => {
	test('returns null when profile file does not exist', async () => {
		const configDir = createTempConfigDir();

		try {
			const result = await readSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBeNull();
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('reads profileId from existing profile file', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'myapp.json'),
				JSON.stringify({profileId: 'uuid-abc-123'}),
				'utf-8',
			);

			const result = await readSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toEqual({profileId: 'uuid-abc-123'});
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('returns null when profile file contains invalid JSON', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'myapp.json'),
				'not-json',
				'utf-8',
			);

			const result = await readSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBeNull();
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('returns null when profile file has no profileId field', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'myapp.json'),
				JSON.stringify({something: 'else'}),
				'utf-8',
			);

			const result = await readSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBeNull();
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('uses default config dir (~/.config/steel) when STEEL_CONFIG_DIR is not set', async () => {
		// Just verify it doesn't throw and returns null for a non-existent profile
		const result = await readSteelProfile(
			`nonexistent-profile-${Date.now()}`,
			{},
		);
		expect(result).toBeNull();
	});
});

describe('writeSteelProfile', () => {
	test('creates profiles directory and writes profile file', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('myapp', 'new-profile-uuid', {
				STEEL_CONFIG_DIR: configDir,
			});

			const filePath = path.join(configDir, 'profiles', 'myapp.json');
			expect(fs.existsSync(filePath)).toBe(true);

			const contents = JSON.parse(fs.readFileSync(filePath, 'utf-8')) as {
				profileId: string;
			};
			expect(contents.profileId).toBe('new-profile-uuid');
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('does not write updatedAt (no timestamp noise in file)', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('myapp', 'some-uuid', {
				STEEL_CONFIG_DIR: configDir,
			});

			const contents = JSON.parse(
				fs.readFileSync(
					path.join(configDir, 'profiles', 'myapp.json'),
					'utf-8',
				),
			) as Record<string, unknown>;
			expect(Object.keys(contents)).toEqual(['profileId']);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('overwrites existing profile', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('myapp', 'first-uuid', {
				STEEL_CONFIG_DIR: configDir,
			});
			await writeSteelProfile('myapp', 'second-uuid', {
				STEEL_CONFIG_DIR: configDir,
			});

			const contents = JSON.parse(
				fs.readFileSync(
					path.join(configDir, 'profiles', 'myapp.json'),
					'utf-8',
				),
			) as {profileId: string};
			expect(contents.profileId).toBe('second-uuid');
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('different profile names are stored in separate files', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('app-a', 'uuid-a', {STEEL_CONFIG_DIR: configDir});
			await writeSteelProfile('app-b', 'uuid-b', {STEEL_CONFIG_DIR: configDir});

			const profilesDir = path.join(configDir, 'profiles');
			const a = JSON.parse(
				fs.readFileSync(path.join(profilesDir, 'app-a.json'), 'utf-8'),
			) as {profileId: string};
			const b = JSON.parse(
				fs.readFileSync(path.join(profilesDir, 'app-b.json'), 'utf-8'),
			) as {profileId: string};

			expect(a.profileId).toBe('uuid-a');
			expect(b.profileId).toBe('uuid-b');
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});
});

describe('listSteelProfiles', () => {
	test('returns empty array when profiles directory does not exist', async () => {
		const configDir = createTempConfigDir();

		try {
			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(result).toEqual([]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('returns empty array when profiles directory is empty', async () => {
		const configDir = createTempConfigDir();

		try {
			fs.mkdirSync(path.join(configDir, 'profiles'), {recursive: true});
			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(result).toEqual([]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('lists all valid profiles', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('app-a', 'uuid-a', {STEEL_CONFIG_DIR: configDir});
			await writeSteelProfile('app-b', 'uuid-b', {STEEL_CONFIG_DIR: configDir});

			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			const sorted = result.sort((a, b) => a.name.localeCompare(b.name));

			expect(sorted).toEqual([
				{name: 'app-a', profileId: 'uuid-a'},
				{name: 'app-b', profileId: 'uuid-b'},
			]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('skips corrupt (invalid JSON) files', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'corrupt.json'),
				'not-json',
				'utf-8',
			);
			await writeSteelProfile('good', 'uuid-good', {
				STEEL_CONFIG_DIR: configDir,
			});

			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(result).toEqual([{name: 'good', profileId: 'uuid-good'}]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('skips files missing profileId field', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'no-id.json'),
				JSON.stringify({something: 'else'}),
				'utf-8',
			);
			await writeSteelProfile('valid', 'uuid-valid', {
				STEEL_CONFIG_DIR: configDir,
			});

			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(result).toEqual([{name: 'valid', profileId: 'uuid-valid'}]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('ignores non-.json files', async () => {
		const configDir = createTempConfigDir();

		try {
			const profilesDir = path.join(configDir, 'profiles');
			fs.mkdirSync(profilesDir, {recursive: true});
			fs.writeFileSync(
				path.join(profilesDir, 'readme.txt'),
				'ignore me',
				'utf-8',
			);
			await writeSteelProfile('myapp', 'uuid-myapp', {
				STEEL_CONFIG_DIR: configDir,
			});

			const result = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(result).toEqual([{name: 'myapp', profileId: 'uuid-myapp'}]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});
});

describe('deleteSteelProfile', () => {
	test('returns false when profile does not exist', async () => {
		const configDir = createTempConfigDir();

		try {
			const result = await deleteSteelProfile('nonexistent', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBe(false);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('deletes the profile file and returns true', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('myapp', 'uuid-123', {
				STEEL_CONFIG_DIR: configDir,
			});
			const filePath = path.join(configDir, 'profiles', 'myapp.json');
			expect(fs.existsSync(filePath)).toBe(true);

			const result = await deleteSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBe(true);
			expect(fs.existsSync(filePath)).toBe(false);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('does not affect other profiles when deleting one', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('app-a', 'uuid-a', {STEEL_CONFIG_DIR: configDir});
			await writeSteelProfile('app-b', 'uuid-b', {STEEL_CONFIG_DIR: configDir});

			await deleteSteelProfile('app-a', {STEEL_CONFIG_DIR: configDir});

			const remaining = await listSteelProfiles({STEEL_CONFIG_DIR: configDir});
			expect(remaining).toEqual([{name: 'app-b', profileId: 'uuid-b'}]);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});

	test('returns false on second delete of same profile', async () => {
		const configDir = createTempConfigDir();

		try {
			await writeSteelProfile('myapp', 'uuid-123', {
				STEEL_CONFIG_DIR: configDir,
			});
			await deleteSteelProfile('myapp', {STEEL_CONFIG_DIR: configDir});

			const result = await deleteSteelProfile('myapp', {
				STEEL_CONFIG_DIR: configDir,
			});
			expect(result).toBe(false);
		} finally {
			fs.rmSync(configDir, {recursive: true, force: true});
		}
	});
});
