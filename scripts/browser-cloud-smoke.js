import fsPromises from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const ANSI_ESCAPE_PATTERN = new RegExp('\\\\u001B\\[[0-9;]*m', 'g');

function stripAnsi(value) {
	return value.replaceAll(ANSI_ESCAPE_PATTERN, '');
}

function normalizeOutput(value) {
	return stripAnsi(value).replaceAll('\r\n', '\n').trim();
}

function parseOutputField(output, keyPattern) {
	const match = output.match(new RegExp(`^${keyPattern}:\\s*(.+)$`, 'im'));
	return match?.[1]?.trim() || null;
}

function extractJsonObject(output) {
	const firstBrace = output.indexOf('{');
	const lastBrace = output.lastIndexOf('}');
	if (firstBrace === -1 || lastBrace === -1 || lastBrace <= firstBrace) {
		return null;
	}

	const candidate = output.slice(firstBrace, lastBrace + 1);
	try {
		return JSON.parse(candidate);
	} catch {
		return null;
	}
}

function parseArguments(argv) {
	let url = 'https://example.com';
	let sessionName = `steel-cloud-smoke-${Date.now()}`;

	for (let index = 0; index < argv.length; index += 1) {
		const argument = argv[index];

		if (argument === '--url') {
			url = argv[index + 1] || url;
			index += 1;
			continue;
		}

		if (argument.startsWith('--url=')) {
			url = argument.slice('--url='.length) || url;
			continue;
		}

		if (argument === '--session') {
			sessionName = argv[index + 1] || sessionName;
			index += 1;
			continue;
		}

		if (argument.startsWith('--session=')) {
			sessionName = argument.slice('--session='.length) || sessionName;
		}
	}

	return {
		url,
		sessionName,
	};
}

function runCommand(command, arguments_, environment, projectRoot) {
	const result = spawnSync(command, arguments_, {
		cwd: projectRoot,
		env: environment,
		encoding: 'utf-8',
	});

	const stdout = result.stdout || '';
	const stderr = result.stderr || '';
	const combined = `${stdout}${stderr}`;

	return {
		status: result.status ?? 1,
		stdout,
		stderr,
		combined,
	};
}

function assertSuccessfulStep(stepName, commandResult) {
	if (commandResult.status === 0) {
		return;
	}

	throw new Error(
		[
			`${stepName} failed with exit code ${commandResult.status}.`,
			commandResult.stdout.trim()
				? `stdout:\n${commandResult.stdout.trim()}`
				: null,
			commandResult.stderr.trim()
				? `stderr:\n${commandResult.stderr.trim()}`
				: null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

async function resolveVendoredRuntime(projectRoot) {
	const manifestPath = path.join(
		projectRoot,
		'dist/vendor/agent-browser/runtime-manifest.json',
	);
	const manifestContent = await fsPromises.readFile(manifestPath, 'utf-8');
	const manifest = JSON.parse(manifestContent);
	const runtimeTarget = `${process.platform}-${process.arch}`;
	const entrypoint = manifest?.platforms?.[runtimeTarget]?.entrypoint;

	if (typeof entrypoint !== 'string' || !entrypoint.trim()) {
		throw new Error(
			`Runtime manifest does not provide an entrypoint for ${runtimeTarget}.`,
		);
	}

	const runtimeHome = path.dirname(manifestPath);
	const runtimePath = path.join(runtimeHome, entrypoint);

	return {
		runtimeHome,
		runtimePath,
	};
}

function getRuntimeCommand(runtimePath) {
	if (
		runtimePath.endsWith('.js') ||
		runtimePath.endsWith('.mjs') ||
		runtimePath.endsWith('.cjs')
	) {
		return {
			command: process.execPath,
			args: [runtimePath],
		};
	}

	return {
		command: runtimePath,
		args: [],
	};
}

function compareParity(stepName, steelResult, runtimeResult) {
	if (steelResult.status !== runtimeResult.status) {
		throw new Error(
			`${stepName} parity mismatch: steel exit=${steelResult.status}, runtime exit=${runtimeResult.status}.`,
		);
	}

	const normalizedSteelOutput = normalizeOutput(steelResult.combined);
	const normalizedRuntimeOutput = normalizeOutput(runtimeResult.combined);
	const normalizedSteelSnapshotOutput = normalizedSteelOutput
		.replaceAll(/@e\d+/g, '@e#')
		.replaceAll(/ref=e\d+/g, 'ref=e#');
	const normalizedRuntimeSnapshotOutput = normalizedRuntimeOutput
		.replaceAll(/@e\d+/g, '@e#')
		.replaceAll(/ref=e\d+/g, 'ref=e#');

	if (normalizedSteelSnapshotOutput === normalizedRuntimeSnapshotOutput) {
		return;
	}

	throw new Error(
		[
			`${stepName} output parity mismatch.`,
			'steel output:',
			normalizedSteelOutput || '(empty)',
			'runtime output:',
			normalizedRuntimeOutput || '(empty)',
		].join('\n'),
	);
}

function resolveSessionFromStartOutput(startOutput, apiKey) {
	const sessionId = parseOutputField(startOutput, 'id');
	const connectUrl = parseOutputField(startOutput, 'connect(?:_|\\s|-)?url');

	return {
		sessionId,
		connectUrl:
			connectUrl ||
			(sessionId && apiKey
				? `wss://connect.steel.dev?apiKey=${apiKey}&sessionId=${sessionId}`
				: null),
	};
}

function resolveSessionFromSessionsCommand(
	sessionName,
	environment,
	projectRoot,
) {
	const sessionsResult = runCommand(
		process.execPath,
		['dist/steel.js', 'browser', 'sessions'],
		environment,
		projectRoot,
	);
	if (sessionsResult.status !== 0) {
		return null;
	}

	const parsed = extractJsonObject(sessionsResult.combined);
	const sessions = Array.isArray(parsed?.sessions) ? parsed.sessions : [];
	const matchingSession = sessions.find(session => {
		return (
			session &&
			typeof session === 'object' &&
			session.name === sessionName &&
			session.live === true
		);
	});
	if (!matchingSession || typeof matchingSession !== 'object') {
		return null;
	}

	const sessionId =
		typeof matchingSession.id === 'string' ? matchingSession.id.trim() : null;
	const connectUrl =
		typeof matchingSession.connectUrl === 'string'
			? matchingSession.connectUrl.trim()
			: null;

	return {
		sessionId: sessionId || null,
		connectUrl: connectUrl || null,
	};
}

async function main() {
	const {url, sessionName} = parseArguments(process.argv.slice(2));
	const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
	const projectRoot = path.resolve(scriptDirectory, '..');

	try {
		await fsPromises.access(path.join(projectRoot, 'dist/steel.js'));
	} catch {
		throw new Error(
			'dist/steel.js is missing. Run `npm run build` before smoke tests.',
		);
	}

	const apiKey = process.env.STEEL_API_KEY?.trim();
	if (!apiKey) {
		console.log(
			'[browser-cloud-smoke] Skipping: STEEL_API_KEY is required for authenticated cloud smoke.',
		);
		return;
	}

	const {runtimeHome, runtimePath} = await resolveVendoredRuntime(projectRoot);
	const runtimeCommand = getRuntimeCommand(runtimePath);

	const baseEnvironment = {
		...process.env,
		STEEL_CLI_SKIP_UPDATE_CHECK: 'true',
		FORCE_COLOR: '0',
		NODE_NO_WARNINGS: '1',
	};

	let shouldStopSession = false;

	try {
		const startResult = runCommand(
			process.execPath,
			['dist/steel.js', 'browser', 'start', '--session', sessionName],
			baseEnvironment,
			projectRoot,
		);
		assertSuccessfulStep('browser start', startResult);
		shouldStopSession = true;

		const normalizedStartOutput = normalizeOutput(startResult.combined);
		let {sessionId, connectUrl} = resolveSessionFromStartOutput(
			normalizedStartOutput,
			apiKey,
		);

		if (!sessionId || !connectUrl) {
			const sessionFromList = resolveSessionFromSessionsCommand(
				sessionName,
				baseEnvironment,
				projectRoot,
			);

			if (!sessionId && sessionFromList?.sessionId) {
				sessionId = sessionFromList.sessionId;
			}

			if (!connectUrl && sessionFromList?.connectUrl) {
				connectUrl = sessionFromList.connectUrl;
			}
		}

		if (!sessionId) {
			throw new Error(
				[
					'Failed to parse session id from `steel browser start` output.',
					'start output:',
					normalizedStartOutput || '(empty)',
				].join('\n'),
			);
		}

		if (!connectUrl) {
			connectUrl = `wss://connect.steel.dev?apiKey=${apiKey}&sessionId=${sessionId}`;
		}

		const steelOpenResult = runCommand(
			process.execPath,
			['dist/steel.js', 'browser', 'open', url, '--session', sessionName],
			baseEnvironment,
			projectRoot,
		);
		const runtimeOpenResult = runCommand(
			runtimeCommand.command,
			[...runtimeCommand.args, 'open', url, '--cdp', connectUrl],
			{
				...baseEnvironment,
				AGENT_BROWSER_HOME: runtimeHome,
			},
			projectRoot,
		);
		compareParity('open', steelOpenResult, runtimeOpenResult);

		const steelSnapshotResult = runCommand(
			process.execPath,
			['dist/steel.js', 'browser', 'snapshot', '-i', '--session', sessionName],
			baseEnvironment,
			projectRoot,
		);
		const runtimeSnapshotResult = runCommand(
			runtimeCommand.command,
			[...runtimeCommand.args, 'snapshot', '-i', '--cdp', connectUrl],
			{
				...baseEnvironment,
				AGENT_BROWSER_HOME: runtimeHome,
			},
			projectRoot,
		);
		compareParity('snapshot -i', steelSnapshotResult, runtimeSnapshotResult);

		console.log(
			`[browser-cloud-smoke] Passed authenticated flow for session ${sessionId}.`,
		);
	} finally {
		if (shouldStopSession) {
			const stopResult = runCommand(
				process.execPath,
				['dist/steel.js', 'browser', 'stop'],
				baseEnvironment,
				projectRoot,
			);
			assertSuccessfulStep('browser stop', stopResult);
		}
	}
}

await main();
