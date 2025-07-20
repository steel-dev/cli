#!/usr/bin/env node

import Pastel from 'pastel';
import React from 'react';
import {render} from 'ink';
import Help from './components/help.js';

// Check if help flag is provided
const helpFlag = process.argv.includes('--help') || process.argv.includes('-h');
const args = process.argv.slice(2).filter(arg => !arg.startsWith('-'));
const command = args.length > 0 ? args.join(' ') : '';

if (helpFlag || !command) {
	const {waitUntilExit} = render(<Help command={command} />);
	await waitUntilExit();
	process.exit(0);
}

const app = new Pastel({
	importMeta: import.meta,
	version: '0.0.1-alpha.7',
});

await app.run();
