#!/usr/bin/env node

import * as React from 'react';
import {Box, Text} from 'ink';
import Spinner from 'ink-spinner';
import zod from 'zod';
import {option} from 'pastel';
import {
	findChromeProfiles,
	isChromeRunning,
	packageChromeProfile,
	updateProfileOnSteel,
	type ChromeProfile,
} from '../../utils/browser/profile-porter.js';
import {
	readSteelProfile,
	validateProfileName,
	writeSteelProfile,
} from '../../utils/browser/lifecycle/profile-store.js';
import {resolveBrowserAuth} from '../../utils/browser/auth.js';
import {DEFAULT_API_PATH} from '../../utils/browser/lifecycle/constants.js';

export const description =
	'Sync a local Chrome profile to an existing Steel profile';

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
				description: 'Chrome profile to sync from (overrides stored source)',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

type Phase =
	| {tag: 'syncing'; chromeProfile: ChromeProfile; step: string}
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

			if (process.platform !== 'darwin') {
				setPhase({
					tag: 'error',
					message: '`steel profile sync` is currently macOS only.',
				});
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

			const chromeProfileDirName = options.from ?? stored.chromeProfile;
			if (!chromeProfileDirName) {
				setPhase({
					tag: 'error',
					message: `No source Chrome profile stored for "${options.name}". Specify one with --from.`,
				});
				return;
			}

			const allProfiles = findChromeProfiles();
			const chromeProfile = allProfiles.find(
				p => p.dirName === chromeProfileDirName,
			);
			if (!chromeProfile) {
				setPhase({
					tag: 'error',
					message: `Chrome profile "${chromeProfileDirName}" not found. Available: ${allProfiles.map(p => p.dirName).join(', ')}`,
				});
				return;
			}

			if (isChromeRunning()) {
				console.error(
					'Warning: Chrome is running. Close it for best results (cookie file may be locked).',
				);
			}

			setPhase({tag: 'syncing', chromeProfile, step: 'Starting...'});

			let zipBuffer: Buffer;
			let cookiesReencrypted: number;

			try {
				({zipBuffer, cookiesReencrypted} = packageChromeProfile(
					chromeProfile.dirName,
					msg => {
						setPhase({tag: 'syncing', chromeProfile, step: msg});
					},
				));
			} catch (error) {
				setPhase({
					tag: 'error',
					message: error instanceof Error ? error.message : String(error),
				});
				return;
			}

			setPhase({
				tag: 'syncing',
				chromeProfile,
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

			if (options.from && options.from !== stored.chromeProfile) {
				await writeSteelProfile(
					options.name,
					stored.profileId,
					process.env,
					options.from,
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

	if (phase.tag === 'syncing') {
		return (
			<Box>
				<Text color="cyan">
					<Spinner type="dots" />
				</Text>
				<Text>
					{' '}
					{phase.chromeProfile.displayName} → {options.name}
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
