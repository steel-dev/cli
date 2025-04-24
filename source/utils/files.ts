import fs from 'fs';
import path from 'path';

const requiredImportsPuppeteerJs = [
	/from ['"]puppeteer-core['"]/,
	/from ['"]puppeteer['"]/,
];
const codePatternsPuppeteerJs = [/puppeteer.launch()/];

const requiredImportsPlaywrightJs = [
	/from ['"]puppeteer-core['"]/,
	/from ['"]puppeteer['"]/,
];
const codePatternsPlaywrightJs = [/launch()/];

const requiredImportsPlaywrightPy = [/from playwright/];
const codePatternsPlaywrightPy = [/launch()/, /playwright/];

const requiredImportsBrowserUsePy = [/from browser_use import Agent/];
const codePatternsBrowserUsePy = [/Agent/];

const requiredImportsOAIComputerUseJs = [/from ['"]openai['"]/];
const codePatternsOAIComputerUseJs = [/new OpenAI/];

const requiredImportsSeleniumPy = [/from selenium/];
const codePatternsSeleniumPy = [/someFunctionName/, /someProperty =/];

const possibleJsImports = [
	{
		name: 'puppeteer',
		imports: requiredImportsPuppeteerJs,
		codePatterns: codePatternsPuppeteerJs,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightJs,
		codePatterns: codePatternsPlaywrightJs,
	},
	{
		name: 'oaiComputerUse',
		imports: requiredImportsOAIComputerUseJs,
		codePatterns: codePatternsOAIComputerUseJs,
	},
];
const possiblePyImports = [
	{
		name: 'browserUse',
		imports: requiredImportsBrowserUsePy,
		codePatterns: codePatternsBrowserUsePy,
	},
	{
		name: 'playwright',
		imports: requiredImportsPlaywrightPy,
		codePatterns: codePatternsPlaywrightPy,
	},
	{
		name: 'selenium',
		imports: requiredImportsSeleniumPy,
		codePatterns: codePatternsSeleniumPy,
	},
];

function searchFileJs(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possibleJsImports) {
		if (
			posImport.imports.some(regex => regex.test(content)) &&
			posImport.codePatterns.some(regex => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			return posImport.name;
		} else return null;
	}
}

function searchFilePy(filePath: string) {
	const content = fs.readFileSync(filePath, 'utf8');
	for (const posImport of possiblePyImports) {
		if (
			posImport.imports.some(regex => regex.test(content)) &&
			posImport.codePatterns.some(regex => regex.test(content))
		) {
			console.log(`✅ Match in: ${filePath}`);
			return;
		}
	}
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
			searchFileJs(fullPath);
		}
	}
}

function walkDirPy(dir: string) {
	const files = fs.readdirSync(dir);
	for (const file of files) {
		const fullPath = path.join(dir, file);
		if (fullPath.endsWith('.py')) {
			searchFilePy(fullPath);
		}
	}
}
