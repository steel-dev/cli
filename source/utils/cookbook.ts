import path from 'path';
import fs from 'fs';

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

	let envContent = fs.readFileSync(envPath, 'utf8');
	const lines = envContent.split('\n');

	let keyFound = false;
	const updatedLines = lines.map(line => {
		if (!line.trim() || line.trim().startsWith('#')) {
			return line;
		}

		const [currentKey] = line.split('=');
		if (currentKey === key) {
			keyFound = true;
			return `${key}=${newValue}`;
		}
		return line;
	});

	if (!keyFound) {
		updatedLines.push(`${key}=${newValue}`);
	}

	fs.writeFileSync(envPath, updatedLines.join('\n'));
}
