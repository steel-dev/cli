import * as os from 'node:os';
import * as path from 'node:path';
import {describe, test, expect, vi, afterEach} from 'vitest';
import {
	SUPPORTED_BROWSERS,
	getBrowserDescriptor,
	getProfileBaseDir,
} from '../../source/utils/browser/profile-porter/browsers';
import type {BrowserId} from '../../source/utils/browser/profile-porter/types';

afterEach(() => {
	vi.restoreAllMocks();
});

describe('SUPPORTED_BROWSERS', () => {
	const allBrowserIds: BrowserId[] = [
		'chrome',
		'edge',
		'brave',
		'arc',
		'opera',
		'vivaldi',
	];

	test('contains all expected browsers', () => {
		for (const id of allBrowserIds) {
			expect(SUPPORTED_BROWSERS[id]).toBeDefined();
			expect(SUPPORTED_BROWSERS[id]!.id).toBe(id);
			expect(SUPPORTED_BROWSERS[id]!.displayName).toBeTruthy();
		}
	});

	test('every browser has at least one profileBaseDir', () => {
		for (const browser of Object.values(SUPPORTED_BROWSERS)) {
			const dirs = Object.values(browser.profileBaseDirs);
			expect(dirs.length).toBeGreaterThan(0);
		}
	});

	test('every browser has at least one processName entry', () => {
		for (const browser of Object.values(SUPPORTED_BROWSERS)) {
			const names = Object.values(browser.processNames);
			expect(names.length).toBeGreaterThan(0);
		}
	});
});

describe('getBrowserDescriptor', () => {
	test('returns correct descriptor for known browser', () => {
		const chrome = getBrowserDescriptor('chrome');
		expect(chrome.id).toBe('chrome');
		expect(chrome.displayName).toBe('Google Chrome');
	});

	test('throws for unknown browser', () => {
		expect(() => getBrowserDescriptor('firefox' as BrowserId)).toThrow(
			'Unknown browser',
		);
	});
});

describe('getProfileBaseDir', () => {
	test('returns macOS path for Chrome', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.chrome, 'darwin');
		expect(dir).toBe(
			path.join(
				os.homedir(),
				'Library',
				'Application Support',
				'Google',
				'Chrome',
			),
		);
	});

	test('returns Linux path for Chrome', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.chrome, 'linux');
		expect(dir).toBe(path.join(os.homedir(), '.config', 'google-chrome'));
	});

	test('returns null for Arc on Linux (unsupported)', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.arc, 'linux');
		expect(dir).toBeNull();
	});

	test('returns null for Arc on Windows (unsupported)', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.arc, 'win32');
		expect(dir).toBeNull();
	});

	test('returns macOS path for Edge', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.edge, 'darwin');
		expect(dir).toBe(
			path.join(
				os.homedir(),
				'Library',
				'Application Support',
				'Microsoft Edge',
			),
		);
	});

	test('returns macOS path for Brave', () => {
		const dir = getProfileBaseDir(SUPPORTED_BROWSERS.brave, 'darwin');
		expect(dir).toBe(
			path.join(
				os.homedir(),
				'Library',
				'Application Support',
				'BraveSoftware',
				'Brave-Browser',
			),
		);
	});
});
