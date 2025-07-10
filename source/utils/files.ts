import fs from 'fs';
import path from 'path';
import {CodeSections} from './types.js';
import {possibleJsImports, possiblePyImports} from './packageConfig.js';

function searchFileJs(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possibleJsImports) {
		if (
			posImport.imports.some((regex: RegExp) => regex.test(content)) &&
			posImport.codePatterns.some((regex: RegExp) => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			posImport.config(filePath);
			return {name: posImport.name, file: filePath};
		}
	}
	return null;
}

function searchFilePy(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possiblePyImports) {
		if (
			posImport.imports.some((regex: RegExp) => regex.test(content)) &&
			posImport.codePatterns.some((regex: RegExp) => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			posImport.config(filePath);
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

export function walkDirPy(dir: string): {
	name: string;
	file: string;
} | null {
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

export function fileParse(filePath: string): CodeSections {
	// I want to grab imports and exports and then anything in between is the body.
	if (!fs.existsSync(filePath)) {
		throw new Error(`File not found: ${filePath}`);
	}

	const ext = path.extname(filePath);
	const isTS = ext === '.ts';
	const isJS = ext === '.js';
	const isPy = ext === '.py';

	if (!isTS && !isPy && !isJS) {
		throw new Error(`Unsupported file type: ${ext}`);
	}

	const lines = fs.readFileSync(filePath, 'utf8').split('\n');

	const imports: string[] = [];
	const exports: string[] = [];
	const body: string[] = [];

	for (const line of lines) {
		const trimmed = line.trim();

		if (isTS) {
			if (trimmed.startsWith('import ')) {
				imports.push(line);
			} else if (trimmed.startsWith('export ')) {
				exports.push(line);
			} else {
				body.push(line);
			}
		} else if (isPy) {
			if (trimmed.startsWith('import ') || trimmed.startsWith('from ')) {
				imports.push(line);
			} else {
				body.push(line); // No concept of 'export' in Python
			}
		}
	}

	return {imports, exports, body};
}

export function readIndentation(line: string | undefined): number | undefined {
	let indentation = 0;
	//@ts-ignore
	if (!line) return;
	for (const char of line) {
		if (char === ' ' || char === '\t') {
			indentation++;
		} else {
			break;
		}
	}
	return indentation;
}

export function indentation(
	line: string | undefined,
	index: number,
	arr: Array<string>,
): number {
	let indentation: number = 0;
	let curIndentation = readIndentation(line);
	if (curIndentation) {
		indentation = curIndentation;
	}
	if (index + 1 < arr.length) {
		let nextIndentation = readIndentation(arr[index + 1]);
		if (nextIndentation && nextIndentation >= indentation) {
			indentation = nextIndentation;
		}
	}
	return indentation;
}
//@ts-ignore
function isPythonIndentationStart(item: string) {
	const controlKeywords = [
		'def',
		'if',
		'for',
		'while',
		'with',
		'try',
		'class',
		'elif',
		'else',
		'except',
		'finally',
	];
	const commentIndex = item.indexOf('#');
	const codePart = commentIndex >= 0 ? item.slice(0, commentIndex) : item;
	const trimmed = codePart.trim();
	return controlKeywords.some(
		keyword =>
			trimmed.startsWith(keyword + ' ') || trimmed.startsWith(keyword + ':'),
	);
}

export function insertCode(
	search: string,
	addedCode: Array<string>,
	body: Array<string>,
	comment: '#' | '//' = '#',
) {
	let indentation: number | undefined;
	let prevIndentation: number | undefined;
	let found: boolean = false;

	for (let index = 0; index < body.length; index++) {
		let item = body[index];
		if (item === undefined) continue;

		let commentIndex: number = Math.min(item.indexOf(comment));
		if (commentIndex === -1) {
			commentIndex = item.length;
		}
		let substring: string = item.substring(0, commentIndex);
		if (substring.includes(search)) {
			found = true;
			console.log(substring);
		}
		// if (found) {
		let curIndentation = readIndentation(item);
		indentation = curIndentation;
		// }
		console.log('PREV:', prevIndentation);
		console.log('CUR:', indentation);
		if (
			found &&
			((indentation === undefined && prevIndentation) ||
				(indentation && prevIndentation))
		) {
			let indentedCode: Array<string> = [];
			if (indentation === undefined && prevIndentation) {
				indentedCode = addedCode.map(
					code => ' '.repeat(prevIndentation!) + code,
				);
			} else if (indentation && prevIndentation < indentation) {
				indentedCode = addedCode.map(
					code => ' '.repeat(prevIndentation!) + code,
				);
			} else if (indentation) {
				indentedCode = addedCode.map(code => ' '.repeat(indentation!) + code);
			}

			body.splice(index + 1, 0, ...indentedCode);
			break;
		}
		// if (indentation) {
		// 	prevIndentation = indentation;
		// }
		if (isPythonIndentationStart(item) && indentation) {
			console.log('UPDATING: ', item);
			prevIndentation = indentation;
		}
	}
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
