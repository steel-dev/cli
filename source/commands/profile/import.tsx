#!/usr/bin/env node

import * as React from 'react';
import {Box, Text} from 'ink';
import Spinner from 'ink-spinner';
import SelectInput from 'ink-select-input';
import zod from 'zod';
import {option} from 'pastel';
import {
	findBrowserProfiles,
	detectInstalledBrowsers,
	isBrowserRunning,
	getBrowserDescriptor,
	getProfileBaseDir,
	packageProfile,
	createKeyProvider,
	uploadProfileToSteel,
	type BrowserDescriptor,
	type BrowserId,
	type BrowserProfile,
} from '../../utils/browser/profile-porter/index.js';
import {
	validateProfileName,
	writeSteelProfile,
} from '../../utils/browser/lifecycle/profile-store.js';
import {resolveBrowserAuth} from '../../utils/browser/auth.js';
import {DEFAULT_API_PATH} from '../../utils/browser/lifecycle/constants.js';
import * as path from 'node:path';

export const description = 'Import a local browser profile into Steel';

export const options = zod.object({
	name: zod.string().describe(
		option({
			description: 'Steel profile name to save as',
		}),
	),
	from: zod
		.string()
		.describe(
			option({
				description:
					'Browser profile to import from (e.g. "Default", "Profile 1")',
			}),
		)
		.optional(),
	browser: zod
		.string()
		.describe(
			option({
				description:
					'Browser to import from (chrome, edge, brave, arc, opera, vivaldi)',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

type Phase =
	| {tag: 'checking'}
	| {
			tag: 'selectingBrowser';
			browsers: BrowserDescriptor[];
	  }
	| {
			tag: 'selecting';
			browser: BrowserDescriptor;
			profiles: BrowserProfile[];
			browserRunning: boolean;
	  }
	| {tag: 'keyPrompt'; browser: BrowserDescriptor}
	| {
			tag: 'importing';
			browser: BrowserDescriptor;
			profile: BrowserProfile;
			step: string;
	  }
	| {tag: 'done'; profileId: string; cookiesReencrypted: number; zipMb: string}
	| {tag: 'error'; message: string};

export default function Import({options}: Props) {
	const [phase, setPhase] = React.useState<Phase>({tag: 'checking'});

	React.useEffect(() => {
		const nameError = validateProfileName(options.name);
		if (nameError) {
			setPhase({tag: 'error', message: nameError});
			return;
		}

		const auth = resolveBrowserAuth(process.env);
		if (!auth.apiKey) {
			setPhase({
				tag: 'error',
				message: 'No API key found. Run `steel login` or set STEEL_API_KEY.',
			});
			return;
		}

		if (options.browser) {
			let browser: BrowserDescriptor;
			try {
				browser = getBrowserDescriptor(options.browser as BrowserId);
			} catch {
				setPhase({
					tag: 'error',
					message: `Unknown browser "${options.browser}". Supported: chrome, edge, brave, arc, opera, vivaldi`,
				});
				return;
			}

			startWithBrowser(browser, auth.apiKey!);
			return;
		}

		// Auto-detect installed browsers
		const installed = detectInstalledBrowsers();
		if (installed.length === 0) {
			setPhase({
				tag: 'error',
				message: 'No supported Chromium browsers found.',
			});
			return;
		}

		if (installed.length === 1) {
			startWithBrowser(installed[0]!, auth.apiKey!);
			return;
		}

		setPhase({tag: 'selectingBrowser', browsers: installed});
	}, []);

	function startWithBrowser(browser: BrowserDescriptor, apiKey: string) {
		const profiles = findBrowserProfiles(browser);
		if (profiles.length === 0) {
			setPhase({
				tag: 'error',
				message: `No ${browser.displayName} profiles found.`,
			});
			return;
		}

		if (options.from) {
			const match = profiles.find(p => p.dirName === options.from);
			if (!match) {
				setPhase({
					tag: 'error',
					message: `Profile "${options.from}" not found in ${browser.displayName}. Available: ${profiles.map(p => p.dirName).join(', ')}`,
				});
				return;
			}
			runImport(browser, match, apiKey);
			return;
		}

		setPhase({
			tag: 'selecting',
			browser,
			profiles,
			browserRunning: isBrowserRunning(browser),
		});
	}

	function runImport(
		browser: BrowserDescriptor,
		profile: BrowserProfile,
		apiKey: string,
	) {
		setPhase({tag: 'importing', browser, profile, step: 'Starting...'});

		(async () => {
			let zipBuffer: Buffer;
			let cookiesReencrypted: number;
			let zipMb: string;

			const baseDir = getProfileBaseDir(browser);
			if (!baseDir) {
				setPhase({
					tag: 'error',
					message: `${browser.displayName} is not supported on this platform.`,
				});
				return;
			}

			try {
				const keyProvider = createKeyProvider(browser);
				({zipBuffer, cookiesReencrypted} = await packageProfile({
					profileDir: path.join(baseDir, profile.dirName),
					keyProvider,
					onProgress: msg => {
						setPhase({tag: 'importing', browser, profile, step: msg});
					},
					onKeyPrompt: () => {
						setPhase({tag: 'keyPrompt', browser});
					},
				}));
				zipMb = (zipBuffer.length / 1024 / 1024).toFixed(1);
			} catch (error) {
				setPhase({
					tag: 'error',
					message: error instanceof Error ? error.message : String(error),
				});
				return;
			}

			setPhase({
				tag: 'importing',
				browser,
				profile,
				step: 'Uploading to Steel...',
			});

			let profileId: string;
			try {
				profileId = await uploadProfileToSteel(
					zipBuffer,
					apiKey,
					DEFAULT_API_PATH,
				);
			} catch (error) {
				setPhase({
					tag: 'error',
					message: error instanceof Error ? error.message : String(error),
				});
				return;
			}

			await writeSteelProfile(
				options.name,
				profileId,
				process.env,
				profile.dirName,
				browser.id,
			);

			setPhase({tag: 'done', profileId, cookiesReencrypted, zipMb});
			process.exit(0);
		})();
	}

	function handleBrowserSelect(item: {value: string}) {
		if (phase.tag !== 'selectingBrowser') return;
		const browser = phase.browsers.find(b => b.id === item.value)!;
		const auth = resolveBrowserAuth(process.env);
		startWithBrowser(browser, auth.apiKey!);
	}

	function handleProfileSelect(item: {value: string}) {
		if (phase.tag !== 'selecting') return;
		const profile = phase.profiles.find(p => p.dirName === item.value)!;
		const auth = resolveBrowserAuth(process.env);
		runImport(phase.browser, profile, auth.apiKey!);
	}

	if (phase.tag === 'checking') {
		return (
			<Box>
				<Text color="cyan">
					<Spinner type="dots" />
				</Text>
				<Text> Checking...</Text>
			</Box>
		);
	}

	if (phase.tag === 'selectingBrowser') {
		const items = phase.browsers.map(b => ({
			label: b.displayName,
			value: b.id,
		}));

		return (
			<Box flexDirection="column" gap={1}>
				<Text bold>Select browser to import from:</Text>
				<SelectInput items={items} onSelect={handleBrowserSelect} />
			</Box>
		);
	}

	if (phase.tag === 'selecting') {
		const items = phase.profiles.map(p => ({
			label: `${p.displayName}  ${p.dirName !== p.displayName ? `(${p.dirName})` : ''}`,
			value: p.dirName,
		}));

		return (
			<Box flexDirection="column" gap={1}>
				{phase.browserRunning && (
					<Box>
						<Text color="yellow">
							{phase.browser.displayName} is running. Close it for best results
							(cookie file may be locked).
						</Text>
					</Box>
				)}
				<Text bold>Select {phase.browser.displayName} profile to import:</Text>
				<SelectInput items={items} onSelect={handleProfileSelect} />
			</Box>
		);
	}

	if (phase.tag === 'keyPrompt') {
		if (process.platform === 'darwin') {
			return (
				<Box>
					<Text color="yellow">
						macOS will ask for your password to read {phase.browser.displayName}
						's cookie encryption key from Keychain.
					</Text>
				</Box>
			);
		}

		return (
			<Box>
				<Text color="yellow">
					Reading {phase.browser.displayName}'s cookie encryption key...
				</Text>
			</Box>
		);
	}

	if (phase.tag === 'importing') {
		return (
			<Box>
				<Text color="cyan">
					<Spinner type="dots" />
				</Text>
				<Text>
					{' '}
					{phase.profile.displayName} → {options.name}
					{'  '}
					<Text dimColor>{phase.step}</Text>
				</Text>
			</Box>
		);
	}

	if (phase.tag === 'done') {
		return (
			<Box flexDirection="column">
				<Box>
					<Text color="green">✔ </Text>
					<Text>
						Imported as <Text bold>{options.name}</Text>
					</Text>
				</Box>
				<Box marginLeft={2} flexDirection="column">
					<Text dimColor>id: {phase.profileId}</Text>
					<Text dimColor>
						cookies: {phase.cookiesReencrypted} re-encrypted · {phase.zipMb} MB
					</Text>
				</Box>
				<Box marginTop={1} flexDirection="column" gap={0}>
					<Text dimColor>steel browser start --profile {options.name}</Text>
					<Text dimColor>
						Add --update-profile to save session changes back to the profile
					</Text>
				</Box>
			</Box>
		);
	}

	// error
	return (
		<Box>
			<Text color="red">✖ </Text>
			<Text>{(phase as {tag: 'error'; message: string}).message}</Text>
		</Box>
	);
}
