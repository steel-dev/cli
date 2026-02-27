import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(scriptDirectory, '..');

const DEFAULT_FILTERS = [];
const DEFAULT_RUNNER_TIMEOUT_MS = 45 * 60 * 1000;
const ANSI_ESCAPE_PATTERN = new RegExp(
	`${String.fromCharCode(27)}\\[[0-9;]*m`,
	'g',
);

function stripAnsi(value) {
	return String(value || '').replace(ANSI_ESCAPE_PATTERN, '');
}

function normalizeInlineText(value) {
	return stripAnsi(value).replaceAll(/\s+/g, ' ').trim();
}

function normalizeStatus(status) {
	switch (status) {
		case 'passed':
			return 'PASS';
		case 'failed':
			return 'FAIL';
		case 'pending':
		case 'skipped':
			return 'SKIP';
		case 'todo':
			return 'TODO';
		default:
			return String(status || 'UNKNOWN').toUpperCase();
	}
}

function inferTestType(filePath) {
	if (filePath.includes('/tests/unit/') || filePath.includes('tests/unit/')) {
		return 'unit';
	}

	if (
		filePath.includes('/tests/integration/') ||
		filePath.includes('tests/integration/')
	) {
		return 'integration';
	}

	return 'other';
}

function formatDuration(durationMs) {
	if (!Number.isFinite(durationMs)) {
		return '-';
	}

	if (durationMs < 1000) {
		return `${Math.round(durationMs)}ms`;
	}

	return `${(durationMs / 1000).toFixed(2)}s`;
}

function truncate(value, maxLength) {
	const normalized = normalizeInlineText(value);
	if (normalized.length <= maxLength) {
		return normalized;
	}

	return `${normalized.slice(0, Math.max(0, maxLength - 1))}…`;
}

function pad(value, width) {
	return truncate(value, width).padEnd(width, ' ');
}

function parseFailureNote(assertion, suite) {
	if (
		Array.isArray(assertion.failureMessages) &&
		assertion.failureMessages.length > 0
	) {
		return normalizeInlineText(assertion.failureMessages[0]);
	}

	if (typeof suite.message === 'string' && suite.message.trim()) {
		return normalizeInlineText(suite.message);
	}

	return '';
}

function toRows(results) {
	const rows = [];

	for (const suite of results.testResults || []) {
		const rawFilePath =
			typeof suite.name === 'string' && suite.name.trim()
				? path.relative(projectRoot, suite.name)
				: '(unknown file)';
		const filePath = rawFilePath.replaceAll('\\', '/');
		const testType = inferTestType(filePath);
		const suiteAssertions = Array.isArray(suite.assertionResults)
			? suite.assertionResults
			: [];

		if (suiteAssertions.length === 0) {
			const suiteDurationMs =
				typeof suite.endTime === 'number' && typeof suite.startTime === 'number'
					? suite.endTime - suite.startTime
					: NaN;
			rows.push({
				type: testType,
				description: path.basename(filePath),
				status: normalizeStatus(suite.status),
				durationMs: suiteDurationMs,
				duration: formatDuration(suiteDurationMs),
				suite: '(suite)',
				file: filePath,
				note: parseFailureNote({}, suite),
			});
			continue;
		}

		for (const assertion of suiteAssertions) {
			const suiteTitle = Array.isArray(assertion.ancestorTitles)
				? assertion.ancestorTitles.join(' > ')
				: '';
			const description =
				typeof assertion.title === 'string' && assertion.title.trim()
					? assertion.title
					: assertion.fullName || '(unnamed test)';

			rows.push({
				type: testType,
				description,
				status: normalizeStatus(assertion.status),
				durationMs:
					typeof assertion.duration === 'number' ? assertion.duration : NaN,
				duration: formatDuration(
					typeof assertion.duration === 'number' ? assertion.duration : NaN,
				),
				suite: suiteTitle || '(suite)',
				file: filePath,
				note: parseFailureNote(assertion, suite),
			});
		}
	}

	return rows;
}

function renderAsciiTable(rows) {
	const columns = [
		{label: '#', key: 'index', width: 3},
		{label: 'Type', key: 'type', width: 11},
		{label: 'Status', key: 'status', width: 6},
		{label: 'Duration', key: 'duration', width: 8},
		{label: 'Description', key: 'description', width: 30},
		{label: 'File', key: 'file', width: 20},
		{label: 'Notes', key: 'note', width: 20},
	];

	const divider = `+${columns
		.map(column => '-'.repeat(column.width + 2))
		.join('+')}+`;
	const header = `| ${columns
		.map(column => pad(column.label, column.width))
		.join(' | ')} |`;

	const body = rows.map((row, index) => {
		const preparedRow = {
			index: String(index + 1),
			...row,
		};
		return `| ${columns
			.map(column => pad(String(preparedRow[column.key] ?? ''), column.width))
			.join(' | ')} |`;
	});

	return [divider, header, divider, ...body, divider].join('\n');
}

function countByStatus(rows) {
	const counts = {
		PASS: 0,
		FAIL: 0,
		SKIP: 0,
		TODO: 0,
		OTHER: 0,
	};

	for (const row of rows) {
		if (row.status in counts) {
			counts[row.status] += 1;
		} else {
			counts.OTHER += 1;
		}
	}

	return counts;
}

function printSummary(results, rows) {
	const counts = countByStatus(rows);
	const startTime =
		typeof results.startTime === 'number' ? new Date(results.startTime) : null;
	const suiteEndTimes = (results.testResults || [])
		.map(suite => suite.endTime)
		.filter(endTime => typeof endTime === 'number');
	const endTime =
		suiteEndTimes.length > 0 ? new Date(Math.max(...suiteEndTimes)) : null;
	const elapsedMs =
		startTime && endTime
			? Math.max(0, endTime.getTime() - startTime.getTime())
			: NaN;

	const slowestRows = [...rows]
		.filter(row => Number.isFinite(row.durationMs))
		.sort((left, right) => right.durationMs - left.durationMs)
		.slice(0, 5);

	console.log('');
	console.log(
		`Summary: total=${rows.length} pass=${counts.PASS} fail=${counts.FAIL} skip=${counts.SKIP} todo=${counts.TODO}`,
	);
	console.log(`Elapsed: ${formatDuration(elapsedMs)}`);
	if (startTime) {
		console.log(`Started: ${startTime.toISOString()}`);
	}

	if (slowestRows.length > 0) {
		console.log('Slowest tests:');
		for (const row of slowestRows) {
			console.log(`- ${row.duration} | ${row.description} (${row.file})`);
		}
	}

	if (counts.SKIP > 0 && !process.env.STEEL_API_KEY) {
		console.log(
			'Note: Some tests were skipped. Set STEEL_API_KEY for authenticated cloud coverage.',
		);
	}
}

function printFailureDetails(rows) {
	const failedRows = rows.filter(row => row.status === 'FAIL');
	if (failedRows.length === 0) {
		return;
	}

	console.log('');
	console.log('Failure details:');
	for (const [index, row] of failedRows.entries()) {
		console.log(`${index + 1}. ${row.description}`);
		console.log(`   file: ${row.file}`);
		console.log(`   suite: ${row.suite}`);
		console.log(`   note: ${row.note || '(no failure note available)'}`);
	}
}

function resolveRunnerTimeoutMs() {
	const rawTimeout = process.env.STEEL_TEST_RUNNER_TIMEOUT_MS;
	if (!rawTimeout) {
		return DEFAULT_RUNNER_TIMEOUT_MS;
	}

	const parsedTimeout = Number.parseInt(rawTimeout, 10);
	if (!Number.isFinite(parsedTimeout) || parsedTimeout <= 0) {
		return DEFAULT_RUNNER_TIMEOUT_MS;
	}

	return parsedTimeout;
}

function main() {
	const filters = process.argv.slice(2);
	const testFilters = filters.length > 0 ? filters : DEFAULT_FILTERS;
	const runnerTimeoutMs = resolveRunnerTimeoutMs();
	const outputFile = path.join(
		os.tmpdir(),
		`vitest-integration-results-${process.pid}-${Date.now()}.json`,
	);

	const vitestArgs = [
		'exec',
		'--',
		'vitest',
		'run',
		...testFilters,
		'--reporter=json',
		'--outputFile',
		outputFile,
	];

	const commandResult = spawnSync('npm', vitestArgs, {
		cwd: projectRoot,
		env: {
			...process.env,
			FORCE_COLOR: '0',
		},
		encoding: 'utf-8',
		timeout: runnerTimeoutMs,
		killSignal: 'SIGKILL',
	});

	const commandStderrParts = [commandResult.stderr || ''];
	if (commandResult.error) {
		commandStderrParts.push(`runner error: ${commandResult.error.message}`);
	}
	if (commandResult.signal) {
		commandStderrParts.push(
			`runner terminated by signal: ${commandResult.signal}`,
		);
	}

	const combinedOutput = [commandResult.stdout || '', ...commandStderrParts]
		.filter(Boolean)
		.join('\n')
		.trim();

	if (commandResult.error?.code === 'ETIMEDOUT') {
		console.error(
			[
				`Vitest run timed out after ${formatDuration(runnerTimeoutMs)}.`,
				'Set STEEL_TEST_RUNNER_TIMEOUT_MS to increase or decrease this limit.',
			].join(' '),
		);
	}

	if (commandResult.status !== 0 && combinedOutput) {
		console.log(combinedOutput);
	}

	if (!fs.existsSync(outputFile)) {
		console.error(
			`Could not read Vitest JSON output at ${outputFile}. The test run may have crashed before results were written.`,
		);
		process.exit(commandResult.status ?? 1);
	}

	let parsedResults;
	try {
		parsedResults = JSON.parse(fs.readFileSync(outputFile, 'utf-8'));
	} catch (error) {
		console.error(`Failed to parse Vitest JSON output at ${outputFile}.`);
		console.error(error instanceof Error ? error.message : String(error));
		process.exit(commandResult.status ?? 1);
	}

	const rows = toRows(parsedResults);
	console.log('');
	console.log('Test Result Table');
	console.log(renderAsciiTable(rows));
	printSummary(parsedResults, rows);
	printFailureDetails(rows);

	try {
		fs.unlinkSync(outputFile);
	} catch {
		// Ignore temporary file cleanup errors.
	}

	if (commandResult.status !== 0 || parsedResults.success === false) {
		process.exit(commandResult.status ?? 1);
	}
}

main();
