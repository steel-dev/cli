import path from 'path';
import fs from 'fs';
import {spawn, ChildProcess} from 'node:child_process';

// Track active child processes to clean them up on exit
const activeProcesses = new Set<ChildProcess>();
let isExiting = false;

// Setup cleanup handlers
function setupProcessCleanup() {
	const cleanup = async (signal?: string) => {
		if (isExiting) return;
		isExiting = true;

		const killPromises = Array.from(activeProcesses).map(child => {
			return new Promise<void>(resolve => {
				if (child.killed) {
					resolve();
					return;
				}

				const forceTimeout = setTimeout(() => {
					if (!child.killed) {
						child.kill('SIGKILL');
					}
					resolve();
				}, 2000);

				child.on('exit', () => {
					clearTimeout(forceTimeout);
					resolve();
				});

				child.kill('SIGTERM');
			});
		});

		try {
			await Promise.all(killPromises);
		} catch (error) {
			console.error('Error during cleanup:', error);
		}

		activeProcesses.clear();

		if (signal === 'SIGINT') {
			process.exit(130);
		} else {
			process.exit(0);
		}
	};

	process.on('SIGINT', () => cleanup('SIGINT'));
	process.on('SIGTERM', () => cleanup('SIGTERM'));
	process.on('beforeExit', () => cleanup());

	process.on('uncaughtException', error => {
		console.error('Uncaught exception:', error);
		cleanup().then(() => process.exit(1));
	});

	process.on('unhandledRejection', reason => {
		console.error('Unhandled rejection:', reason);
		cleanup().then(() => process.exit(1));
	});
}

let cleanupInitialized = false;
if (!cleanupInitialized) {
	setupProcessCleanup();
	cleanupInitialized = true;
}

export function copyDir(srcDir: string, destDir: string) {
	fs.mkdirSync(destDir, {recursive: true});
	for (const file of fs.readdirSync(srcDir)) {
		const srcFile = path.resolve(srcDir, file);
		const destFile = path.resolve(destDir, file);
		copy(srcFile, destFile);
	}
}
export function copy(src: string, dest: string) {
	const stat = fs.statSync(src);
	if (stat.isDirectory()) {
		copyDir(src, dest);
	} else {
		fs.copyFileSync(src, dest);
	}
}
export const write = (
	file: string,
	targetDir: string,
	templateDir: string,
	cwd: string,
	content?: string,
) => {
	const renameFiles: Record<string, string | undefined> = {
		_gitignore: '.gitignore',
	};
	const root = path.join(cwd, targetDir);
	const targetPath = path.join(root, renameFiles[file] ?? file);
	if (content) {
		fs.writeFileSync(targetPath, content);
	} else {
		copy(path.join(templateDir, file), targetPath);
	}
};

export function isValidProjectName(projectName: string) {
	return /^(?:@[a-z\d\-*~][a-z\d\-*._~]*\/)?[a-z\d\-~][a-z\d\-._~]*$/.test(
		projectName,
	);
}

export function toValidProjectName(projectName: string) {
	return projectName
		.trim()
		.toLowerCase()
		.replace(/\s+/g, '-')
		.replace(/^[._]/, '')
		.replace(/[^a-z\d\-~]+/g, '-');
}

export function updateEnvVariable(
	directory: string,
	key: string,
	newValue: string,
) {
	const envPath = path.resolve(directory, '.env');

	const envContent = fs.readFileSync(envPath, 'utf8');
	const lines = envContent.split('\n');

	let keyFound = false;
	const updatedLines = lines.map(line => {
		// if line is empty or starts with a comment or new value is empty, don't update it
		if (!line.trim() || line.trim().startsWith('#')) {
			return line;
		}
		const [currentKey] = line.split('=');
		if (currentKey === key) {
			keyFound = true;
			if (newValue.trim() === '') return line;
			// if new value has spaces, wrap it in quotes
			if (newValue.includes(' ')) {
				newValue = `"${newValue}"`;
			}
			// if new value has special characters, escape them
			if (newValue.includes('\\')) {
				newValue = newValue.replace(/\\/g, '\\\\');
			}
			return `${key}=${newValue}`;
		}
		return line;
	});

	if (!keyFound) {
		updatedLines.push(`${key}=${newValue}`);
	}

	fs.writeFileSync(envPath, updatedLines.join('\n'));
}

export function runCommand(command: string, cwd: string): Promise<void> {
	return new Promise((resolve, reject) => {
		if (isExiting) {
			reject(new Error('Process is exiting'));
			return;
		}

		const child = spawn(command, {
			cwd,
			shell: true,
			stdio: 'ignore',
			detached: false,
		});

		activeProcesses.add(child);

		child.on('exit', code => {
			activeProcesses.delete(child);

			if (code === 0) {
				resolve();
			} else {
				reject(new Error(`Process exited with code ${code}`));
			}
		});

		child.on('error', err => {
			activeProcesses.delete(child);
			reject(err);
		});
	});
}

export function runCommandWithOutput(
	command: string,
	cwd: string,
	onOutput?: (data: string) => void,
): Promise<string> {
	return new Promise((resolve, reject) => {
		if (isExiting) {
			reject(new Error('Process is exiting'));
			return;
		}

		let output = '';
		const child = spawn(command, {
			cwd,
			shell: true,
			stdio: ['pipe', 'pipe', 'pipe'],
			detached: false,
		});

		activeProcesses.add(child);

		// Stream stdout
		child.stdout?.on('data', data => {
			const text = data.toString();
			output += text;
			if (onOutput) {
				onOutput(text);
			}
		});

		// Stream stderr
		child.stderr?.on('data', data => {
			const text = data.toString();
			output += text;
			if (onOutput) {
				onOutput(text);
			}
		});

		child.on('exit', code => {
			activeProcesses.delete(child);

			if (code === 0) {
				resolve(output);
			} else {
				reject(new Error(`Process exited with code ${code}: ${output}`));
			}
		});

		child.on('error', err => {
			activeProcesses.delete(child);
			reject(err);
		});
	});
}
