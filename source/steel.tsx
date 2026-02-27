#!/usr/bin/env -S node --no-warnings=ExperimentalWarning

import Pastel from 'pastel';
import React from 'react';
import {Box, Text, render} from 'ink';
import Help from './components/help.js';
import {
	checkAndUpdate,
	getCurrentVersion,
	setGlobalUpdateInfo,
	subscribeToUpdateState,
	type UpdateState,
} from './utils/update.js';
import Spinner from 'ink-spinner';
import {
	filterSteelGlobalFlags,
	getBrowserPassthroughArgv,
	isBrowserHelpAlias,
	isBrowserCommand,
	resolveBrowserDispatchTarget,
} from './utils/browser/routing.js';
import {
	BrowserAdapterError,
	runBrowserPassthrough,
} from './utils/browser/adapter.js';

const cliArgv = process.argv.slice(2);

// Check for version flag first
const versionFlag = cliArgv.includes('--version') || cliArgv.includes('-v');
if (versionFlag) {
	console.log(getCurrentVersion());
	process.exit(0);
}

if (isBrowserHelpAlias(cliArgv)) {
	const {waitUntilExit} = render(<Help command="browser" />);
	await waitUntilExit();
	process.exit(0);
}

const browserDispatchTarget = resolveBrowserDispatchTarget(cliArgv);

if (browserDispatchTarget === 'passthrough') {
	try {
		const passthroughArgv = getBrowserPassthroughArgv(cliArgv);
		const exitCode = await runBrowserPassthrough(passthroughArgv);
		process.exit(exitCode);
	} catch (error) {
		if (error instanceof BrowserAdapterError) {
			console.error(error.message);
		} else {
			console.error('Failed to execute browser passthrough command.');
		}

		process.exit(1);
	}
}

// Check if help flag is provided
const helpFlag = cliArgv.includes('--help') || cliArgv.includes('-h');
const args = filterSteelGlobalFlags(cliArgv).filter(
	arg => !arg.startsWith('-'),
);
const command = args.length > 0 ? args.join(' ') : '';

// Skip update check for certain commands or flags
const skipUpdateCommands = ['update', 'help'];
const skipUpdateCheck =
	helpFlag ||
	skipUpdateCommands.includes(command) ||
	isBrowserCommand(cliArgv) ||
	cliArgv.includes('--no-update-check') ||
	process.env.STEEL_CLI_SKIP_UPDATE_CHECK === 'true' ||
	process.env.CI === 'true' || // Skip in CI environments
	process.env.NODE_ENV === 'test'; // Skip in test environments

// Handle help commands first (before any update checks)
if (helpFlag || !command || command === 'help') {
	const {waitUntilExit} = render(
		<Help command={command === 'help' ? '' : command} />,
	);
	await waitUntilExit();
	process.exit(0);
}

// Component to show update progress
function UpdateProgress() {
	const [updateState, setUpdateState] = React.useState<UpdateState>({
		status: 'idle',
		message: '',
	});
	const [shouldProceed, setShouldProceed] = React.useState(false);

	React.useEffect(() => {
		const unsubscribe = subscribeToUpdateState(state => {
			setUpdateState(state);

			if (state.status === 'complete') {
				if (state.versionInfo?.hasUpdate) {
					setGlobalUpdateInfo(state.versionInfo || null);
					setShouldProceed(true);
				} else {
					setGlobalUpdateInfo(state.versionInfo || null);
					setShouldProceed(true);
				}
			} else if (state.status === 'error') {
				setShouldProceed(true);
			}
		});

		// Start the update check
		async function performUpdateCheck() {
			try {
				await checkAndUpdate({
					silent: true,
					autoUpdate: true,
					reactMode: true,
				});
			} catch {
				setShouldProceed(true);
			}
		}

		performUpdateCheck();

		// Cleanup subscription on unmount
		return unsubscribe;
	}, []);

	if (
		updateState.status === 'idle' ||
		updateState.status === 'checking' ||
		updateState.status === 'updating'
	) {
		return (
			<Box marginBottom={1}>
				<Text>
					<Spinner type="dots" /> {updateState.message}
				</Text>
			</Box>
		);
	}

	if (updateState.status === 'complete' && updateState.versionInfo?.hasUpdate) {
		return (
			<Box marginBottom={1}>
				<Text>✅ {updateState.message}</Text>
			</Box>
		);
	}

	if (updateState.status === 'error') {
		return (
			<Box marginBottom={1}>
				<Text color="red">❌ {updateState.message}</Text>
				{updateState.error && <Text color="gray"> ({updateState.error})</Text>}
			</Box>
		);
	}

	if (shouldProceed) {
		return <PastelApp />;
	}

	return null;
}

// Component to render the Pastel app
function PastelApp() {
	React.useEffect(() => {
		async function runPastel() {
			// Filter out global flags and update process.argv
			const originalArgv = process.argv;
			process.argv = [
				process.argv[0],
				process.argv[1],
				...filterSteelGlobalFlags(process.argv.slice(2)),
			];

			const app = new Pastel({
				importMeta: import.meta,
				version: getCurrentVersion(),
			});

			try {
				await app.run();
			} finally {
				// Restore original argv
				process.argv = originalArgv;
			}
		}
		runPastel();
	}, []);

	return null;
}

// Main execution
if (!skipUpdateCheck) {
	// Render the update progress component which will handle the flow
	const {waitUntilExit} = render(<UpdateProgress />);
	await waitUntilExit();
} else {
	// Filter out global flags and update process.argv
	const originalArgv = process.argv;
	process.argv = [
		process.argv[0],
		process.argv[1],
		...filterSteelGlobalFlags(process.argv.slice(2)),
	];

	const app = new Pastel({
		importMeta: import.meta,
		version: getCurrentVersion(),
	});

	try {
		await app.run();
	} finally {
		// Restore original argv
		process.argv = originalArgv;
	}
}
