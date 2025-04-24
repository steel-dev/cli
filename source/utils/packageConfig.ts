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
const codePatternsSeleniumPy = [/webdriver/];

export const possibleJsImports = [
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
		name: 'OpenAI ComputerUse',
		imports: requiredImportsOAIComputerUseJs,
		codePatterns: codePatternsOAIComputerUseJs,
	},
];
export const possiblePyImports = [
	{
		name: 'brwoser-use',
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
