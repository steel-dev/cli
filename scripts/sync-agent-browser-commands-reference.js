#!/usr/bin/env node

import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';

const PINNED_COMMIT = 'b59dc4c82c0583b60849d110f27982fac85f1a07';
const SOURCE_URL = `https://raw.githubusercontent.com/vercel-labs/agent-browser/${PINNED_COMMIT}/skills/agent-browser/references/commands.md`;
const SOURCE_API_URL = `https://api.github.com/repos/vercel-labs/agent-browser/contents/skills/agent-browser/references/commands.md?ref=${PINNED_COMMIT}`;
const SOURCE_CACHE_FILE =
	'docs/references/upstream/agent-browser-commands.source.md';
const OUTPUT_FILE = 'docs/references/steel-browser-commands.md';
const execFileAsync = promisify(execFile);

function normalizeLineEndings(value) {
	return value.replaceAll('\r\n', '\n');
}

function applyTransform(content) {
	let transformed = content;

	// Rewrite command-prefix usage while avoiding runtime-specific path/config text.
	transformed = transformed.replaceAll(/\bagent-browser\s+/g, 'steel browser ');

	// Keep heading explicit about the transformed context.
	if (transformed.startsWith('# Command Reference')) {
		transformed = transformed.replace(
			'# Command Reference',
			'# Command Reference (Steel Browser Adaptation)',
		);
	}

	return transformed;
}

function buildHeader() {
	const generatedAt = new Date().toISOString();

	return [
		'# Steel Browser Commands (Synced + Transformed)',
		'',
		'> Generated file. Do not edit manually.',
		`> Source: ${SOURCE_URL}`,
		`> Pinned commit: \`${PINNED_COMMIT}\``,
		`> Generated at: ${generatedAt}`,
		'',
		'## Notes',
		'',
		'- Upstream `agent-browser` command-prefix usage is transformed to `steel browser` for migration-friendly examples.',
		'- Runtime-specific env vars and config names (for example `AGENT_BROWSER_*`) are preserved as upstream runtime details.',
		'- Steel-native lifecycle commands (`steel browser start|stop|sessions|live`) are documented in `./steel-browser.md`.',
		'',
	].join('\n');
}

async function fetchSource() {
	try {
		const response = await fetch(SOURCE_URL);
		if (!response.ok) {
			throw new Error(
				`Failed to download source reference (${response.status} ${response.statusText}).`,
			);
		}

		const sourceText = await response.text();
		return normalizeLineEndings(sourceText);
	} catch {
		// Network may be blocked for Node fetch in some environments; fallback to curl.
		try {
			const {stdout} = await execFileAsync('curl', ['-sL', SOURCE_URL], {
				maxBuffer: 10 * 1024 * 1024,
			});

			if (stdout.trim()) {
				return normalizeLineEndings(stdout);
			}
		} catch {
			// Try GitHub API content endpoint as fallback when raw host is unavailable.
		}

		const {stdout: apiResponse} = await execFileAsync(
			'curl',
			['-sL', SOURCE_API_URL],
			{
				maxBuffer: 10 * 1024 * 1024,
			},
		);
		const parsed = JSON.parse(apiResponse);
		const encodedContent = parsed?.content;
		const encoding = parsed?.encoding;

		if (typeof encodedContent !== 'string' || encoding !== 'base64') {
			throw new Error(
				'Failed to download source reference via GitHub API fallback.',
			);
		}

		const decodedContent = Buffer.from(encodedContent, 'base64').toString(
			'utf-8',
		);
		return normalizeLineEndings(decodedContent);
	}
}

async function run() {
	const currentFile = fileURLToPath(import.meta.url);
	const scriptDirectory = path.dirname(currentFile);
	const projectRoot = path.resolve(scriptDirectory, '..');
	const sourceCachePath = path.join(projectRoot, SOURCE_CACHE_FILE);
	const outputPath = path.join(projectRoot, OUTPUT_FILE);
	const forceCache = process.argv.includes('--from-cache');

	let source;

	if (forceCache) {
		source = normalizeLineEndings(await fs.readFile(sourceCachePath, 'utf-8'));
	} else {
		try {
			source = await fetchSource();
			await fs.mkdir(path.dirname(sourceCachePath), {recursive: true});
			await fs.writeFile(sourceCachePath, source, 'utf-8');
		} catch (error) {
			try {
				source = normalizeLineEndings(
					await fs.readFile(sourceCachePath, 'utf-8'),
				);
				console.warn(
					`[references] Remote fetch failed; using cached source at ${path.relative(projectRoot, sourceCachePath)}.`,
				);
			} catch {
				throw new Error(
					[
						error instanceof Error ? error.message : String(error),
						`No cached source found at ${path.relative(projectRoot, sourceCachePath)}.`,
						'Seed the cache with curl or rerun where outbound network is available.',
					].join(' '),
				);
			}
		}
	}

	const transformed = applyTransform(source);
	const output = `${buildHeader()}\n${transformed}`;

	await fs.mkdir(path.dirname(outputPath), {recursive: true});
	await fs.writeFile(outputPath, output, 'utf-8');

	console.log(
		`[references] Wrote transformed commands reference to ${path.relative(projectRoot, outputPath)}.`,
	);
}

await run();
