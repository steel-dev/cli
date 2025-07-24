import fs from 'fs';
import path from 'path';
import {fileURLToPath} from 'url';
import {CACHE_DIR} from './constants.js';
import {runCommand} from './forge.js';
import type {Template} from './types.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const REGISTRY_BASE_URL = 'https://registry.steel-edge.net';
const MANIFEST_PATH = path.resolve(__dirname, '../../manifest.json');

type ManifestGroup = {
	id: string;
	title: string;
	accentColor: string;
	category: string;
	description: string;
	flags: string[];
};

type ManifestExample = {
	slug: string;
	id: string;
	title: string;
	accentColor: string;
	category: string;
	stack: string;
	description: string;
	flags: string[];
	directory: string;
	language: string;
	template: string;
	groupId?: string;
};

type Manifest = {
	name: string;
	description: string;
	version: string;
	groups: ManifestGroup[];
	examples: ManifestExample[];
};

export function loadManifest(): Manifest {
	if (!fs.existsSync(MANIFEST_PATH)) {
		throw new Error(`Manifest file not found at ${MANIFEST_PATH}`);
	}

	const manifestContent = fs.readFileSync(MANIFEST_PATH, 'utf-8');
	return JSON.parse(manifestContent);
}

export function convertManifestToTemplates(manifest: Manifest): Template[] {
	return manifest.examples
		.filter(example => example.flags.includes('cli'))
		.map(example => {
			const template: Template = {
				alias: example.slug,
				label: example.title,
				value: example.id,
				language: mapLanguage(example.language, example.stack),
				category: example.category,
				accentColor: example.accentColor,
				groupId: example.groupId,
				command: createCommand(example.id),
			};

			template.env = getEnvironmentVariables(example);
			template.depCommands = getDependencyCommands(example);
			template.runCommand = getRunCommand(example);
			template.displayRunCommands = getDisplayRunCommands(example);

			return template;
		});
}

function mapLanguage(language: string, stack: string): string {
	if (language === 'typescript') return 'TS';
	if (language === 'javascript') return 'JS';
	if (language === 'python') return 'PY';
	return language.toUpperCase();
}

function createCommand(id: string): string {
	return id
		.split('-')
		.filter(part => part !== 'steel' && part !== 'starter')
		.join('-');
}

function getEnvironmentVariables(
	example: ManifestExample,
): {value: string; label: string; required?: boolean}[] {
	const baseEnv: {value: string; label: string; required?: boolean}[] = [
		{value: 'STEEL_API_KEY', label: 'Steel API key'},
		{value: 'STEEL_CONNECT_URL', label: 'Steel Connect URL'},
		{value: 'STEEL_API_URL', label: 'Steel API URL'},
	];

	if (
		example.category === 'AI_AGENTS' ||
		example.category === 'AI_AUTOMATION'
	) {
		if (example.id.includes('oai') || example.id.includes('openai')) {
			baseEnv.push({
				value: 'OPENAI_API_KEY',
				label: 'OpenAI API key',
				required: true,
			});
			baseEnv.push({value: 'TASK', label: 'Task for the agent'});
		}
		if (example.id.includes('claude') || example.id.includes('anthropic')) {
			baseEnv.push({
				value: 'ANTHROPIC_API_KEY',
				label: 'Anthropic API key',
				required: true,
			});
			baseEnv.push({value: 'TASK', label: 'Task for the agent'});
		}
		if (example.id.includes('stagehand')) {
			baseEnv.push({
				value: 'OPENAI_API_KEY',
				label: 'OpenAI API key',
				required: true,
			});
		}
	}

	return baseEnv;
}

function getDependencyCommands(example: ManifestExample) {
	return (options: {depsDir: string}) => {
		if (example.stack === 'python') {
			return [
				`python3 -m venv ${options.depsDir}`,
				`${options.depsDir}/bin/pip install -r requirements.txt`,
			];
		} else {
			return [
				`mkdir -p ${options.depsDir}`,
				`cp package.json ${options.depsDir}/package.json`,
				`cd ${options.depsDir} && npm install --no-audit --silent`,
			];
		}
	};
}

function getRunCommand(example: ManifestExample) {
	return (options: {depsDir: string}) => {
		if (example.stack === 'python') {
			return `${options.depsDir}/bin/python3 -u main.py`;
		} else if (example.language === 'typescript') {
			const nodePath = `NODE_PATH=${options.depsDir}/node_modules`;
			const compilerOptions = JSON.stringify({
				baseUrl: '.',
				paths: {
					'*': [`${options.depsDir}/node_modules/*`],
				},
			});
			return `${nodePath} npx ts-node --compiler-options '${compilerOptions}' index.ts`;
		} else {
			return `NODE_PATH=${options.depsDir}/node_modules node index.js`;
		}
	};
}

function getDisplayRunCommands(example: ManifestExample) {
	return (options: {directory: string; packageManager?: string}) => {
		if (example.stack === 'python') {
			return [`cd ${options.directory}`, `python main.py`];
		} else if (example.language === 'typescript') {
			return [
				`cd ${options.directory}`,
				`${options.packageManager || 'npm'} run build`,
				`node dist/index.js`,
			];
		} else {
			return [`cd ${options.directory}`, `node index.js`];
		}
	};
}

export async function downloadTemplate(
	templatePath: string,
	version: string,
): Promise<string> {
	const templateName = path.basename(templatePath, '.tar.gz');
	const cacheKey = `${templateName}-${version}`;
	const cachedPath = path.join(CACHE_DIR, cacheKey);

	if (fs.existsSync(cachedPath)) {
		return cachedPath;
	}

	const downloadUrl = `${REGISTRY_BASE_URL}/${templatePath}?v=${version}`;
	const tempDir = path.join(CACHE_DIR, `temp-${Date.now()}`);
	const tarPath = path.join(tempDir, `${templateName}.tar.gz`);

	fs.mkdirSync(tempDir, {recursive: true});
	fs.mkdirSync(cachedPath, {recursive: true});

	try {
		await runCommand(`curl -L -o "${tarPath}" "${downloadUrl}"`, tempDir);
		await runCommand(
			`tar -xzf "${path.basename(tarPath)}" -C "${cachedPath}"`,
			tempDir,
		);

		fs.rmSync(tempDir, {recursive: true, force: true});

		return cachedPath;
	} catch (error) {
		if (fs.existsSync(tempDir)) {
			fs.rmSync(tempDir, {recursive: true, force: true});
		}
		if (fs.existsSync(cachedPath)) {
			fs.rmSync(cachedPath, {recursive: true, force: true});
		}
		throw error;
	}
}

export function getTemplateDirectory(
	example: ManifestExample,
	manifest: Manifest,
): Promise<string> {
	return downloadTemplate(example.template, manifest.version);
}
