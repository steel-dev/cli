#!/usr/bin/env node

import Pastel from 'pastel';
import React from 'react';
import {render} from 'ink';
import Help from './components/help.js';
import {checkAndUpdate, getCurrentVersion} from './utils/update.js';

// Check for version flag first
const versionFlag =
	process.argv.includes('--version') || process.argv.includes('-v');
if (versionFlag) {
	console.log(getCurrentVersion());
	process.exit(0);
}

// Check if help flag is provided
const helpFlag = process.argv.includes('--help') || process.argv.includes('-h');
const args = process.argv.slice(2).filter(arg => !arg.startsWith('-'));
const command = args.length > 0 ? args.join(' ') : '';

// Skip update check for certain commands or flags
const skipUpdateCommands = ['update', 'help'];
const skipUpdateCheck =
	helpFlag ||
	skipUpdateCommands.includes(command) ||
	process.argv.includes('--no-update-check') ||
	process.env.STEEL_CLI_SKIP_UPDATE_CHECK === 'true' ||
	process.env.CI === 'true' || // Skip in CI environments
	process.env.NODE_ENV === 'test'; // Skip in test environments

if (helpFlag || !command) {
	const {waitUntilExit} = render(<Help command={command} />);
	await waitUntilExit();
	process.exit(0);
}

// Perform update check with timeout (non-blocking)
if (!skipUpdateCheck) {
	try {
		// Set a timeout to avoid blocking the CLI
		const updateCheckPromise = checkAndUpdate({
			silent: true,
			autoUpdate: false,
		});

		const timeoutPromise = new Promise(resolve => {
			setTimeout(() => resolve(null), 3000); // 3 second timeout
		});

		// Race between update check and timeout
		const result = await Promise.race([updateCheckPromise, timeoutPromise]);

		// If result is null, the timeout won fired
		if (result === null) {
			console.debug('Update check timed out');
		}
	} catch (error) {
		// Silently fail - don't block the CLI if update check fails
		console.debug('Update check failed:', error);
	}
}

const app = new Pastel({
	importMeta: import.meta,
	version: getCurrentVersion(),
});

await app.run();
