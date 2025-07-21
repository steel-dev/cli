#!/usr/bin/env node

import React from 'react';
import {Text} from 'ink';
import Spinner from 'ink-spinner';
import zod from 'zod';
import {option} from 'pastel';
import {checkAndUpdate, VersionInfo} from '../utils/update.js';
import Callout from '../components/callout.js';

export const description = 'Update Steel CLI to the latest version';

export const options = zod.object({
	force: zod
		.boolean()
		.describe(
			option({
				description: 'Force update even if already on latest version',
				alias: 'f',
			}),
		)
		.optional(),
	check: zod
		.boolean()
		.describe(
			option({
				description: 'Only check for updates without installing',
				alias: 'c',
			}),
		)
		.optional(),
});

export type Options = zod.infer<typeof options>;

type Props = {
	options: Options;
};

export default function Update({options}: Props) {
	const [loading, setLoading] = React.useState(true);
	const [versionInfo, setVersionInfo] = React.useState<VersionInfo | null>(
		null,
	);
	const [error, setError] = React.useState<string | null>(null);

	React.useEffect(() => {
		async function performUpdate() {
			try {
				const info = await checkAndUpdate({
					force: options.force,
					autoUpdate: !options.check,
					silent: false,
					reactMode: true,
				});
				setVersionInfo(info);
			} catch (err) {
				setError(err instanceof Error ? err.message : 'Unknown error occurred');
			} finally {
				setLoading(false);
			}
		}

		performUpdate();
	}, [options.force, options.check]);

	if (loading) {
		return (
			<Callout variant="info" title="Checking for Updates">
				<Text>
					<Spinner type="dots" /> Checking for updates...
				</Text>
			</Callout>
		);
	}

	if (error) {
		return (
			<Callout variant="failed" title="Update Check Failed">
				{error}
			</Callout>
		);
	}

	if (!versionInfo) {
		return (
			<Callout variant="failed" title="Update Check Failed">
				Could not check for updates
			</Callout>
		);
	}

	if (options.check) {
		if (versionInfo.hasUpdate) {
			return (
				<Callout variant="warning" title="Update Available">
					<Text>
						Current: v{versionInfo.current} {'\n'}
					</Text>
					<Text>
						Latest: v{versionInfo.latest} {'\n'}
					</Text>
					<Text>{'\n'}</Text>
					<Text>
						💡 Run `steel update` to update to the latest version {'\n'}
					</Text>
					{/* {versionInfo.changelog && (
						<>
							<Text>{'\n'}</Text>
							<Text>📝 What&apos;s new: {'\n'}</Text>
							<Text>{versionInfo.changelog}</Text>
						</>
					)} */}
				</Callout>
			);
		} else {
			return (
				<Callout variant="success" title="Up to Date">
					<Text>Current version: v{versionInfo.current}</Text>
					<Text>{'\n'}</Text>
					<Text>You&apos;re already on the latest version!</Text>
				</Callout>
			);
		}
	}

	if (!versionInfo.hasUpdate && !options.force) {
		return (
			<Callout variant="info" title="Already Up to Date">
				<Text>Current version: v{versionInfo.current}</Text>
				<Text>{'\n'}</Text>
				<Text>You&apos;re already on the latest version!</Text>
			</Callout>
		);
	}

	// If we get here, an update was performed or attempted
	return (
		<Callout variant="success" title="Update Completed">
			<Text>
				🚀 Updated from v{versionInfo.current} to v{versionInfo.latest}
			</Text>
			{/* {versionInfo.changelog && (
				<>
					<Text>{'\n'}</Text>
					<Text>📝 What&apos;s new: {'\n'}</Text>
					<Text>{versionInfo.changelog}</Text>
				</>
			)} */}
		</Callout>
	);
}
