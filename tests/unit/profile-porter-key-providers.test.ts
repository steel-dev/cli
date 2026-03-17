import {describe, test, expect, vi, afterEach} from 'vitest';
import {
	deriveKey,
	deriveTargetKey,
} from '../../source/utils/browser/profile-porter/crypto';
import type {BrowserDescriptor} from '../../source/utils/browser/profile-porter/types';

// We test the createKeyProvider factory through its public interface
// by mocking child_process.exec

const mockBrowser: BrowserDescriptor = {
	id: 'chrome',
	displayName: 'Google Chrome',
	profileBaseDirs: {
		darwin: 'Google/Chrome',
		linux: 'google-chrome',
	},
	processNames: {
		darwin: ['Google Chrome'],
	},
	keychainService: {
		darwin: 'Chrome Safe Storage',
		linux: 'chrome',
	},
};

afterEach(() => {
	vi.restoreAllMocks();
	vi.resetModules();
});

describe('createKeyProvider', () => {
	test('throws for unsupported platform', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		expect(() =>
			createKeyProvider(mockBrowser, 'freebsd' as NodeJS.Platform),
		).toThrow('Unsupported platform');
	});

	test('macOS provider returns correct algorithm types', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		const provider = createKeyProvider(mockBrowser, 'darwin');
		expect(provider.sourceAlgorithm.type).toBe('aes-128-cbc');
		expect(provider.targetAlgorithm.type).toBe('aes-128-cbc');
	});

	test('macOS provider throws if no keychain service configured', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		const browserNoKeychain: BrowserDescriptor = {
			...mockBrowser,
			keychainService: {},
		};
		expect(() => createKeyProvider(browserNoKeychain, 'darwin')).toThrow(
			'no macOS Keychain service configured',
		);
	});

	test('linux provider returns correct algorithm types', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		const provider = createKeyProvider(mockBrowser, 'linux');
		expect(provider.sourceAlgorithm.type).toBe('aes-128-cbc');
		expect(provider.targetAlgorithm.type).toBe('aes-128-cbc');
	});

	test('windows provider returns correct algorithm types', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		const winBrowser: BrowserDescriptor = {
			...mockBrowser,
			profileBaseDirs: {win32: 'Google/Chrome/User Data'},
		};
		const provider = createKeyProvider(winBrowser, 'win32');
		expect(provider.sourceAlgorithm.type).toBe('aes-256-gcm');
		expect(provider.targetAlgorithm.type).toBe('aes-128-cbc');
	});

	test('target key is always the peanuts key', async () => {
		const {createKeyProvider} = await import(
			'../../source/utils/browser/profile-porter/key-providers'
		);
		const macProvider = createKeyProvider(mockBrowser, 'darwin');
		const linuxProvider = createKeyProvider(mockBrowser, 'linux');

		const expectedTargetKey = deriveTargetKey();
		expect(macProvider.getTargetKey().equals(expectedTargetKey)).toBe(true);
		expect(linuxProvider.getTargetKey().equals(expectedTargetKey)).toBe(true);
	});
});
