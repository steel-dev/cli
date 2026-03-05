#!/usr/bin/env node

import * as React from 'react';
import {Box, Text} from 'ink';
import Spinner from 'ink-spinner';
import SelectInput from 'ink-select-input';
import zod from 'zod';
import {option} from 'pastel';
import {
	findChromeProfiles,
	isChromeRunning,
	packageChromeProfile,
	uploadProfileToSteel,
	type ChromeProfile,
} from '../../utils/browser/profile-porter.js';
import {writeSteelProfile} from '../../utils/browser/lifecycle/profile-store.js';
import {resolveBrowserAuth} from '../../utils/browser/auth.js';
import {DEFAULT_API_PATH} from '../../utils/browser/lifecycle/constants.js';

export const description =
	'Import a local Chrome profile into Steel (macOS only)';

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
					'Chrome profile to import from (e.g. "Default", "Profile 1")',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

type Phase =
	| {tag: 'checking'}
	| {tag: 'selecting'; profiles: ChromeProfile[]; chromeRunning: boolean}
	| {tag: 'importing'; chromeProfile: ChromeProfile; step: string}
	| {tag: 'done'; profileId: string; cookiesReencrypted: number; zipMb: string}
	| {tag: 'error'; message: string};

export default function Import({options}: Props) {
	const [phase, setPhase] = React.useState<Phase>({tag: 'checking'});

	React.useEffect(() => {
		if (process.platform !== 'darwin') {
			setPhase({
				tag: 'error',
				message: '`steel profile import` is currently macOS only.',
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

		const profiles = findChromeProfiles();
		if (profiles.length === 0) {
			setPhase({
				tag: 'error',
				message: 'No Chrome profiles found.',
			});
			return;
		}

		if (options.from) {
			const match = profiles.find(p => p.dirName === options.from);
			if (!match) {
				setPhase({
					tag: 'error',
					message: `Chrome profile "${options.from}" not found. Available: ${profiles.map(p => p.dirName).join(', ')}`,
				});
				return;
			}

			runImport(match, auth.apiKey!);
			return;
		}

		// No --from: show picker
		setPhase({
			tag: 'selecting',
			profiles,
			chromeRunning: isChromeRunning(),
		});
	}, []);

	function runImport(chromeProfile: ChromeProfile, apiKey: string) {
		setPhase({tag: 'importing', chromeProfile, step: 'Starting...'});

		(async () => {
			let zipBuffer: Buffer;
			let cookiesReencrypted: number;
			let zipMb: string;

			try {
				({zipBuffer, cookiesReencrypted} = packageChromeProfile(
					chromeProfile.dirName,
					msg => {
						setPhase({
							tag: 'importing',
							chromeProfile,
							step: msg,
						});
					},
				));
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
				chromeProfile,
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

			await writeSteelProfile(options.name, profileId, process.env);

			setPhase({tag: 'done', profileId, cookiesReencrypted, zipMb});
			process.exit(0);
		})();
	}

	function handleSelect(item: {value: string}) {
		if (phase.tag !== 'selecting') return;
		const profile = phase.profiles.find(p => p.dirName === item.value)!;
		const auth = resolveBrowserAuth(process.env);
		runImport(profile, auth.apiKey!);
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

	if (phase.tag === 'selecting') {
		const items = phase.profiles.map(p => ({
			label: `${p.displayName}  ${p.dirName !== p.displayName ? `(${p.dirName})` : ''}`,
			value: p.dirName,
		}));

		return (
			<Box flexDirection="column" gap={1}>
				{phase.chromeRunning && (
					<Box>
						<Text color="yellow">
							⚠ Chrome is running. Close it for best results (cookie file may
							be locked).
						</Text>
					</Box>
				)}
				<Text bold>Select Chrome profile to import:</Text>
				<SelectInput items={items} onSelect={handleSelect} />
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
