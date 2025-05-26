#!/usr/bin/env node

import fs from 'fs';
import {walkDirJs, walkDirPy} from '../utils/files.js';
// import {spawn} from 'node:child_process';

export const description =
	'BETA (DO NOT USE IF YOU CARE ABOUT YOUR PROJECT): Integrates the Steel CLI into your project';

export default function integrate() {
	// I need to first determine if I am in a python/javscript/typescript environment
	// Next I need to figure out what type of tool I am using to connecty with the steel browser
	// For sake of brevity, lets just say that we will only look in the current directory where this is run for files that end with .py, .js, .jsx, .ts, or .tsx
	// This way, this wont be distructive on a large scraping code base

	const workingDir = process.cwd();
	// console.log(workingDir);
	const files = fs.readdirSync(workingDir);
	let environment;

	for (const file of files) {
		if (file.endsWith('.py')) {
			environment = 'python';
		} else if (file.endsWith('.js') || file.endsWith('.jsx')) {
			environment = 'javascript';
		} else if (file.endsWith('.ts') || file.endsWith('.tsx')) {
			environment = 'typescript';
		}
	}

	if (!environment) {
		console.error('Could not determine environment');
		process.exit(1);
	}

	if (environment === 'javascript' || environment === 'typescript') {
		let packageManager = '';

		for (const file of files) {
			//console.log(file);
			switch (file) {
				case 'package-lock.json':
					packageManager = 'npm';
					break;
				case 'pnpm-lock.yaml':
					packageManager = 'pnpm';
					break;
				case 'bun.lockb':
					packageManager = 'bun';
					break;
				case 'bun.lock':
					packageManager = 'bun';
					break;
				case 'yarn.lock':
					packageManager = 'yarn';
					break;
				default:
					packageManager = 'npm';
					break;
			}
		}
		console.log(`Found ${packageManager} as your package manager!`);
		// spawn(packageManager, [
		// 	packageManager === 'npm' ? 'install' : 'add',
		// 	'steel-sdk',
		// ]);
		const found = walkDirJs(workingDir);
		if (found) {
			console.log(`Found ${found.name}!`);
		} else {
			console.log(`No automation found in ${workingDir}!`);
		}
	} else if (environment === 'python') {
		console.log('Found Python as your environment!');
		// spawn('pip', ['install', 'steel-sdk']);
		const found = walkDirPy(workingDir);
		if (found) {
			console.log(`Found ${found.name}!`);
		} else {
			console.log(`No automation found in ${workingDir}!`);
		}
	}
}
