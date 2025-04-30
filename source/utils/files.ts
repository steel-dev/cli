import fs from 'fs';
import path from 'path';
import {possibleJsImports, possiblePyImports} from './packageConfig.js';

function searchFileJs(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possibleJsImports) {
		if (
			posImport.imports.some(regex => regex.test(content)) &&
			posImport.codePatterns.some(regex => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			return {name: posImport.name, file: filePath};
		}
	}
	return null;
}

function searchFilePy(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possiblePyImports) {
		if (
			posImport.imports.some(regex => regex.test(content)) &&
			posImport.codePatterns.some(regex => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			return {name: posImport.name, file: filePath};
		}
	}
	return null;
}

export function walkDirJs(dir: string) {
	const files = fs.readdirSync(dir);
	for (const file of files) {
		const fullPath = path.join(dir, file);
		if (
			fullPath.endsWith('.ts') ||
			fullPath.endsWith('.tsx') ||
			fullPath.endsWith('.js') ||
			fullPath.endsWith('.jsx')
		) {
			return searchFileJs(fullPath);
		}
	}
	return null;
}

export function walkDirPy(dir: string) {
	const files = fs.readdirSync(dir);
	for (const file of files) {
		const fullPath = path.join(dir, file);
		if (fullPath.endsWith('.py')) {
			return searchFilePy(fullPath);
		}
	}
	return null;
}

export function appendToTopofFile(file: string, string: string) {
	console.log('Appending to ', file);

	// Read the existing file contents asynchronously
	fs.readFile(file, 'utf8', (err, data) => {
		if (err) {
			console.error('Error reading file:', err);
			return;
		}

		// Combine the new text with the existing contents
		const updatedContent = string + data;

		// Write the updated content back to the file asynchronously
		fs.writeFile(file, updatedContent, 'utf8', err => {
			if (err) {
				console.error('Error writing file:', err);
				return;
			}
			console.log(`${file}: File updated successfully!`);
		});
	});
}

export function wrapStringinFile(
	file: string,
	beforeString: string,
	stringToFind: string,
	afterString: string,
) {
	try {
		// Read the existing file contents
		const data = fs.readFileSync(file, 'utf8');

		// Find the position of the search string
		const index = data.lastIndexOf(stringToFind);
		if (index === -1) {
			console.log('Search string not found in the file.');
			return;
		}

		// Insert the text before and after the search string
		const before = data.slice(0, index);
		const target = data.slice(index, index + stringToFind.length);
		const after = data.slice(index + stringToFind.length);

		const updatedContent = before + beforeString + target + afterString + after;

		// Write the updated content back to the file
		fs.writeFileSync(file, updatedContent, 'utf8');
		console.log(`${file}: File updated successfully!`);
	} catch (err) {
		console.error('Error reading or writing file:', err);
	}
}

export function replaceString(file: string, string: string, newString: string) {
	try {
		// Read the existing file contents
		const data = fs.readFileSync(file, 'utf8');

		// Find the position of the search string
		const index = data.lastIndexOf(string);
		if (index === -1) {
			console.log('Search string not found in the file.');
			return;
		}

		// Insert the text before and after the search string
		const before = data.slice(0, index);
		const after = data.slice(index + string.length);

		const updatedContent = before + newString + after;

		// Write the updated content back to the file
		fs.writeFileSync(file, updatedContent, 'utf8');
		console.log(`${file}: File updated successfully!`);
	} catch (err) {
		console.error('Error reading or writing file:', err);
	}
}

export function ensureDirectoryExists(dirPath: string) {
	try {
		fs.mkdirSync(dirPath, {recursive: true});
		console.log(`Directory created or already exists: ${dirPath}`);
	} catch (error: any) {
		console.error(`Error creating directory: ${error.message}`);
	}
}

export function ensureAndAppendFile(filePath: string, content: string) {
	try {
		// Check if the file exists
		fs.accessSync(filePath);
		console.log('File exists. Proceeding to write to it.');
	} catch (error: any) {
		if (error.code === 'ENOENT') {
			console.log('File does not exist. It will be created.');
		} else {
			throw error; // Re-throw if it's not a "file does not exist" error
		}
	}

	try {
		// Append to the file (creates it if it does not exist)
		fs.appendFileSync(filePath, content, 'utf8');
		console.log(`${filePath}: Data appended successfully.`);
	} catch (err: any) {
		console.error('Error appending to file:', err.message);
	}
}
