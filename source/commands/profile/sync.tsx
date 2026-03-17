#!/usr/bin/env node

import * as React from 'react';
import * as path from 'node:path';
import {Box, Text} from 'ink';
import Spinner from 'ink-spinner';
import zod from 'zod';
import {option} from 'pastel';
import {
	findBrowserProfiles,
	isBrowserRunning,
	getBrowserDescriptor,
	getProfileBaseDir,
	packageProfile,
	createKeyProvider,
	updateProfileOnSteel,
	type BrowserDescriptor,
	type BrowserId,
	type BrowserProfile,
} from '../../utils/browser/profile-porter/index.js';
import {
	readSteelProfile,
	validateProfileName,
	writeSteelProfile,
} from '../../utils/browser/lifecycle/profile-store.js';
import {resolveBrowserAuth} from '../../utils/browser/auth.js';
import {DEFAULT_API_PATH} from '../../utils/browser/lifecycle/constants.js';

export const description =
	'Sync a local browser profile to an existing Steel profile';

export const options = zod.object({
	name: zod.string().describe(
		option({
			description: 'Steel profile name to sync',
		}),
	),
	from: zod
		.string()
		.describe(
			option({
				description: 'Browser profile to sync from (overrides stored source)',
			}),
		)
		.optional(),
	browser: zod
		.string()
		.describe(
			option({
				description: 'Browser to sync from (overrides stored browser)',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

type Phase =
	| {tag: 'keyPrompt'; browser: BrowserDescriptor}
	| {
			tag: 'syncing';
			browser: BrowserDescriptor;
			profile: BrowserProfile;
			step: string;
	  }
	| {tag: 'done'; cookiesReencrypted: number; zipMb: string}
	| {tag: 'error'; message: string};

export default function Sync({options}: Props) {
	const [phase, setPhase] = React.useState<Phase | null>(null);

	React.useEffect(() => {
		(async () => {
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

			const stored = await readSteelProfile(options.name, process.env);
			if (!stored) {
				setPhase({
					tag: 'error',
					message: `Profile "${options.name}" not found. Run \`steel profile import --name ${options.name}\` first.`,
				});
				return;
			}

			// Resolve browser: --browser flag > stored browser > default 'chrome'
			const browserId = (options.browser ??
				stored.browser ??
				'chrome') as BrowserId;
			let browser: BrowserDescriptor;
			try {
				browser = getBrowserDescriptor(browserId);
			} catch {
				setPhase({
					tag: 'error',
					message: `Unknown browser "${browserId}". Supported: chrome, edge, brave, arc, opera, vivaldi`,
				});
				return;
			}

			const profileDirName = options.from ?? stored.chromeProfile;
			if (!profileDirName) {
				setPhase({
					tag: 'error',
					message: `No source browser profile stored for "${options.name}". Specify one with --from.`,
				});
				return;
			}

			const allProfiles = findBrowserProfiles(browser);
			const profile = allProfiles.find(p => p.dirName === profileDirName);
			if (!profile) {
				setPhase({
					tag: 'error',
					message: `${browser.displayName} profile "${profileDirName}" not found. Available: ${allProfiles.map(p => p.dirName).join(', ')}`,
				});
				return;
			}

			if (isBrowserRunning(browser)) {
				console.error(
					`Warning: ${browser.displayName} is running. Close it for best results (cookie file may be locked).`,
				);
			}

			const baseDir = getProfileBaseDir(browser);
			if (!baseDir) {
				setPhase({
					tag: 'error',
					message: `${browser.displayName} is not supported on this platform.`,
				});
				return;
			}

			setPhase({tag: 'syncing', browser, profile, step: 'Starting...'});

			let zipBuffer: Buffer;
			let cookiesReencrypted: number;

			try {
				const keyProvider = createKeyProvider(browser);
				({zipBuffer, cookiesReencrypted} = await packageProfile({
					profileDir: path.join(baseDir, profile.dirName),
					keyProvider,
					onProgress: msg => {
						setPhase({tag: 'syncing', browser, profile, step: msg});
					},
					onKeyPrompt: () => {
						setPhase({tag: 'keyPrompt', browser});
					},
				}));
			} catch (error) {
				setPhase({
					tag: 'error',
					message: error instanceof Error ? error.message : String(error),
				});
				return;
			}

			setPhase({
				tag: 'syncing',
				browser,
				profile,
				step: 'Uploading to Steel...',
			});

			try {
				await updateProfileOnSteel(
					stored.profileId,
					zipBuffer,
					auth.apiKey,
					DEFAULT_API_PATH,
				);
			} catch (error) {
				setPhase({
					tag: 'error',
					message: error instanceof Error ? error.message : String(error),
				});
				return;
			}

			// Update stored profile if browser or source changed
			if (
				options.from !== stored.chromeProfile ||
				browserId !== (stored.browser ?? 'chrome')
			) {
				await writeSteelProfile(
					options.name,
					stored.profileId,
					process.env,
					options.from ?? stored.chromeProfile,
					browserId,
				);
			}

			const zipMb = (zipBuffer.length / 1024 / 1024).toFixed(1);
			setPhase({tag: 'done', cookiesReencrypted, zipMb});
			process.exit(0);
		})();
	}, []);

	if (phase === null) {
		return (
			<Box>
				<Text color="cyan">
					<Spinner type="dots" />
				</Text>
				<Text> Checking...</Text>
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

	if (phase.tag === 'syncing') {
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
						Synced <Text bold>{options.name}</Text>
					</Text>
				</Box>
				<Box marginLeft={2}>
					<Text dimColor>
						{phase.cookiesReencrypted} cookies re-encrypted · {phase.zipMb} MB
					</Text>
				</Box>
			</Box>
		);
	}

	return (
		<Box>
			<Text color="red">✖ </Text>
			<Text>{phase.message}</Text>
		</Box>
	);
}
