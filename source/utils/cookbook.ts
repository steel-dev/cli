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
