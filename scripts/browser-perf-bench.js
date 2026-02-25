import fsPromises from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const SUPPORTED_RUNTIME_TARGETS = new Set([
	'darwin-arm64',
	'darwin-x64',
	'linux-arm64',
	'linux-x64',
	'win32-arm64',
	'win32-x64',
]);

const SCENARIO_DEFINITIONS = {
	coldOpen: {
		commandArgs: ['browser', 'open', 'https://example.com'],
	},
	warmSnapshot: {
		commandArgs: ['browser', 'snapshot', '-i', '--cdp', 'ws://127.0.0.1:9222'],
	},
	warmClick: {
		commandArgs: ['browser', 'click', 'ref-1', '--cdp', 'ws://127.0.0.1:9222'],
	},
};

function parseArguments() {
	return {
		enforce:
			process.argv.includes('--enforce') ||
			process.env.STEEL_BROWSER_PERF_ENFORCE === 'true',
		json:
			process.argv.includes('--json') ||
			process.env.STEEL_BROWSER_PERF_JSON === 'true',
	};
}

function computeMedian(values) {
	const sorted = [...values].sort((left, right) => left - right);
	if (sorted.length === 0) {
		return 0;
	}

	const middleIndex = Math.floor(sorted.length / 2);
	if (sorted.length % 2 === 0) {
		return (sorted[middleIndex - 1] + sorted[middleIndex]) / 2;
	}

	return sorted[middleIndex];
}

function formatMillis(value) {
	return `${value.toFixed(1)}ms`;
}

function runCli(projectRoot, cliArgs, environment) {
	return new Promise((resolve, reject) => {
		const startedAt = process.hrtime.bigint();
		let stdout = '';
		let stderr = '';

		const child = spawn(process.execPath, ['dist/steel.js', ...cliArgs], {
			cwd: projectRoot,
			env: environment,
		});

		child.stdout.on('data', chunk => {
			stdout += chunk;
		});
		child.stderr.on('data', chunk => {
			stderr += chunk;
		});

		child.once('error', reject);
		child.once('close', code => {
			const completedAt = process.hrtime.bigint();
			const durationMs = Number(completedAt - startedAt) / 1_000_000;

			if (code !== 0) {
				reject(
					new Error(
						[
							`Command failed: steel ${cliArgs.join(' ')}`,
							`exit=${code}`,
							stdout.trim() ? `stdout:\n${stdout.trim()}` : '',
							stderr.trim() ? `stderr:\n${stderr.trim()}` : '',
						]
							.filter(Boolean)
							.join('\n'),
					),
				);
				return;
			}

			resolve(durationMs);
		});
	});
}

async function createFixtureRuntimeScript(tempDirectory) {
	const runtimePath = path.join(tempDirectory, 'perf-runtime-fixture.mjs');

	await fsPromises.writeFile(
		runtimePath,
		[
			'process.stdout.write("");',
			'process.stderr.write("");',
			'process.exit(0);',
			'',
		].join('\n'),
		'utf-8',
	);

	return runtimePath;
}

function createJsonResponse(payload) {
	return JSON.stringify(payload);
}

async function startMockSessionApiServer() {
	const sessions = new Map();
	let sessionCounter = 0;

	const server = http.createServer((request, response) => {
		const method = request.method || 'GET';
		const requestUrl = new URL(request.url || '/', 'http://127.0.0.1');
		const pathname = requestUrl.pathname;

		if (method === 'POST' && pathname === '/v1/sessions') {
			sessionCounter += 1;
			const sessionId = `perf-session-${sessionCounter}`;
			const payload = {
				id: sessionId,
				status: 'live',
				websocketUrl: `ws://127.0.0.1:9222/${sessionId}`,
			};
			sessions.set(sessionId, payload);

			response.statusCode = 201;
			response.setHeader('Content-Type', 'application/json');
			response.end(createJsonResponse(payload));
			return;
		}

		if (method === 'GET' && pathname === '/v1/sessions') {
			response.statusCode = 200;
			response.setHeader('Content-Type', 'application/json');
			response.end(createJsonResponse({sessions: [...sessions.values()]}));
			return;
		}

		if (method === 'GET' && pathname.startsWith('/v1/sessions/')) {
			const sessionId = pathname.split('/').at(-1);
			const session = sessionId ? sessions.get(sessionId) : null;

			if (!session) {
				response.statusCode = 404;
				response.setHeader('Content-Type', 'application/json');
				response.end(createJsonResponse({message: 'Session not found'}));
				return;
			}

			response.statusCode = 200;
			response.setHeader('Content-Type', 'application/json');
			response.end(createJsonResponse(session));
			return;
		}

		if (
			method === 'POST' &&
			pathname.startsWith('/v1/sessions/') &&
			pathname.endsWith('/release')
		) {
			const parts = pathname.split('/');
			const sessionId = parts[3];
			if (sessionId) {
				sessions.delete(sessionId);
			}

			response.statusCode = 200;
			response.setHeader('Content-Type', 'application/json');
			response.end(createJsonResponse({ok: true}));
			return;
		}

		response.statusCode = 404;
		response.setHeader('Content-Type', 'application/json');
		response.end(createJsonResponse({message: 'Not found'}));
	});

	await new Promise((resolve, reject) => {
		server.once('error', reject);
		server.listen(0, '127.0.0.1', resolve);
	});

	const address = server.address();
	if (!address || typeof address === 'string') {
		throw new Error('Failed to resolve mock API server port.');
	}

	return {
		baseUrl: `http://127.0.0.1:${address.port}/v1`,
		close: () =>
			new Promise((resolve, reject) => {
				server.close(error => {
					if (error) {
						reject(error);
						return;
					}

					resolve();
				});
			}),
	};
}

async function ensureDistExists(projectRoot) {
	const distEntrypoint = path.join(projectRoot, 'dist/steel.js');
	try {
		await fsPromises.access(distEntrypoint);
	} catch {
		throw new Error(
			'dist/steel.js is missing. Run `npm run build` before benchmarking.',
		);
	}
}

function evaluateBudgets(metrics, budgets) {
	const violations = [];

	for (const [scenarioName, summary] of Object.entries(metrics)) {
		const scenarioBudget = budgets.scenarios[scenarioName];
		if (!scenarioBudget) {
			continue;
		}

		const medianMs = summary.medianMs;
		if (
			typeof scenarioBudget.maxMs === 'number' &&
			medianMs > scenarioBudget.maxMs
		) {
			violations.push(
				`${scenarioName}: median ${formatMillis(medianMs)} exceeded max ${formatMillis(scenarioBudget.maxMs)}.`,
			);
		}

		if (
			typeof scenarioBudget.baselineMs === 'number' &&
			typeof scenarioBudget.maxRegressionPct === 'number'
		) {
			const allowedMs =
				scenarioBudget.baselineMs * (1 + scenarioBudget.maxRegressionPct / 100);
			if (medianMs > allowedMs) {
				violations.push(
					`${scenarioName}: median ${formatMillis(medianMs)} exceeded regression threshold ${formatMillis(allowedMs)} (baseline=${formatMillis(scenarioBudget.baselineMs)}, maxRegressionPct=${scenarioBudget.maxRegressionPct}%).`,
				);
			}
		}
	}

	return violations;
}

async function main() {
	const {enforce, json} = parseArguments();
	const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(scriptDirectory, '..');
	const runtimeTarget = `${process.platform}-${process.arch}`;

	if (!SUPPORTED_RUNTIME_TARGETS.has(runtimeTarget)) {
		console.log(
			`[browser-perf] Unsupported runtime target ${runtimeTarget}; skipping benchmark.`,
		);
		return;
	}

	await ensureDistExists(projectRoot);

	const budgetPath = path.join(scriptDirectory, 'browser-perf-budgets.json');
	const budgetContent = await fsPromises.readFile(budgetPath, 'utf-8');
	const budgets = JSON.parse(budgetContent);
	const runCount =
		Number(process.env.STEEL_BROWSER_PERF_RUNS || budgets.runs || 5) || 5;

	const tempDirectory = await fsPromises.mkdtemp(
		path.join(os.tmpdir(), 'steel-browser-perf-'),
	);
	const runtimePath = await createFixtureRuntimeScript(tempDirectory);
	const mockApiServer = await startMockSessionApiServer();
	const metrics = {};

	try {
		for (const [scenarioName, scenarioConfig] of Object.entries(
			SCENARIO_DEFINITIONS,
		)) {
			const durations = [];

			for (let runIndex = 0; runIndex < runCount; runIndex++) {
				const runConfigDirectory = path.join(
					tempDirectory,
					'config',
					`${scenarioName}-${runIndex}`,
				);
				await fsPromises.mkdir(runConfigDirectory, {recursive: true});

				const durationMs = await runCli(
					projectRoot,
					scenarioConfig.commandArgs,
					{
						...process.env,
						STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
						STEEL_API_KEY: 'perf-api-key',
						STEEL_API_URL: mockApiServer.baseUrl,
						STEEL_BROWSER_RUNTIME_BIN: runtimePath,
						STEEL_CONFIG_DIR: runConfigDirectory,
						FORCE_COLOR: '0',
					},
				);
				durations.push(durationMs);
			}

			metrics[scenarioName] = {
				medianMs: computeMedian(durations),
				minMs: Math.min(...durations),
				maxMs: Math.max(...durations),
				samples: durations,
			};
		}

		const violations = evaluateBudgets(metrics, budgets);

		if (json) {
			console.log(
				JSON.stringify(
					{
						metrics,
						violations,
						runCount,
						enforce,
					},
					null,
					2,
				),
			);
		} else {
			console.log(`[browser-perf] Runs per scenario: ${runCount}`);
			for (const [scenarioName, summary] of Object.entries(metrics)) {
				console.log(
					`[browser-perf] ${scenarioName}: median=${formatMillis(summary.medianMs)} min=${formatMillis(summary.minMs)} max=${formatMillis(summary.maxMs)}`,
				);
			}
		}

		if (violations.length > 0) {
			for (const violation of violations) {
				console.error(`[browser-perf] ${violation}`);
			}

			if (enforce) {
				process.exitCode = 1;
			}
		}
	} finally {
		await mockApiServer.close();
		await fsPromises.rm(tempDirectory, {recursive: true, force: true});
	}
}

await main();
