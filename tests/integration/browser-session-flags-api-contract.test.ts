import {
	assertSuccessfulStep,
	createCloudHarness,
	createLegacyRunBrowserCommand,
	extractSessionId,
	stripAnsi,
	withCloudSession,
} from './harness';

type UnknownRecord = Record<string, unknown>;

function assertConnectUrlIsRedacted(commandOutput: string): void {
	const normalizedOutput = stripAnsi(commandOutput);
	const connectUrlMatch = normalizedOutput.match(
		/(?:^|\n)connect_url:\s*([^\s]+)/,
	);
	if (!connectUrlMatch?.[1]) {
		return;
	}

	expect(connectUrlMatch[1]).not.toMatch(
		/[?&](?:apiKey|api_key|token|access_token)=(?!REDACTED\b)[^&]+/i,
	);
}

function asRecord(value: unknown): UnknownRecord | null {
	if (!value || typeof value !== 'object' || Array.isArray(value)) {
		return null;
	}

	return value as UnknownRecord;
}

async function getSessionById(
	sessionId: string,
	apiKey: string,
): Promise<UnknownRecord> {
	for (let attempt = 0; attempt < 5; attempt++) {
		const response = await fetch(
			`https://api.steel.dev/v1/sessions/${sessionId}`,
			{
				headers: {
					'Steel-Api-Key': apiKey,
					'Content-Type': 'application/json',
				},
			},
		);
		const bodyText = await response.text();

		if (response.status === 404 && attempt < 4) {
			await new Promise(resolve => {
				setTimeout(resolve, 400);
			});
			continue;
		}

		if (!response.ok) {
			throw new Error(
				[
					`Failed to fetch session ${sessionId} (${response.status}).`,
					bodyText.trim() ? `body:\n${bodyText.trim()}` : null,
				]
					.filter(Boolean)
					.join('\n'),
			);
		}

		if (!bodyText.trim()) {
			throw new Error(`Session details response for ${sessionId} was empty.`);
		}

		const parsedBody = JSON.parse(bodyText) as unknown;
		const topLevel = asRecord(parsedBody);
		if (!topLevel) {
			throw new Error(
				`Session details for ${sessionId} were not a JSON object.`,
			);
		}

		const nestedSession = asRecord(topLevel['session']);
		return nestedSession || topLevel;
	}

	throw new Error(
		`Failed to fetch session ${sessionId} because it remained unavailable.`,
	);
}

async function releaseSessionById(
	sessionId: string,
	apiKey: string,
): Promise<void> {
	const response = await fetch(
		`https://api.steel.dev/v1/sessions/${sessionId}/release`,
		{
			method: 'POST',
			headers: {
				'Steel-Api-Key': apiKey,
				'Content-Type': 'application/json',
			},
		},
	);

	if (response.ok) {
		return;
	}

	const bodyText = await response.text();
	throw new Error(
		[
			`Failed to release session ${sessionId} (${response.status}).`,
			bodyText.trim() ? `body:\n${bodyText.trim()}` : null,
		]
			.filter(Boolean)
			.join('\n'),
	);
}

describe('browser session flag mapping contract', () => {
	const harness = createCloudHarness(import.meta.url);
	const cloudTest = harness.cloudTest;

	cloudTest(
		'maps --stealth and --proxy flags to expected Steel Sessions API fields',
		async () => {
			await withCloudSession(
				harness,
				{
					configDirectoryPrefix: 'steel-browser-session-flags-contract-',
					sessionNamePrefix: 'steel-browser-session-flags-contract',
				},
				async ({environment, projectRoot, runCommand}) => {
					const runBrowserCommand = createLegacyRunBrowserCommand(runCommand);
					const createdSessionIds = new Set<string>();

					try {
						const baselineName = `steel-browser-flags-baseline-${Date.now()}`;
						const baselineStartResult = runBrowserCommand(
							['start', '--session', baselineName],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('baseline browser start', baselineStartResult);
						assertConnectUrlIsRedacted(baselineStartResult.output);

						const baselineSessionId = extractSessionId(
							baselineStartResult.output,
						);
						createdSessionIds.add(baselineSessionId);

						const baselineSession = await getSessionById(
							baselineSessionId,
							harness.apiKey!,
						);
						const baselineStealthConfig = asRecord(
							baselineSession['stealthConfig'],
						);

						expect(baselineSession['proxySource']).not.toBe('external');
						expect(baselineStealthConfig?.['humanizeInteractions']).not.toBe(
							true,
						);
						expect(baselineStealthConfig?.['autoCaptchaSolving']).not.toBe(
							true,
						);
						expect(baselineSession['solveCaptcha']).not.toBe(true);

						const stopBaselineResult = runBrowserCommand(
							['stop'],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('baseline browser stop', stopBaselineResult);

						const flaggedName = `steel-browser-flags-stealth-proxy-${Date.now()}`;
						const flaggedStartResult = runBrowserCommand(
							[
								'start',
								'--session',
								flaggedName,
								'--stealth',
								'--proxy',
								'http://127.0.0.1:8080',
							],
							environment,
							projectRoot,
						);
						assertSuccessfulStep('flagged browser start', flaggedStartResult);
						assertConnectUrlIsRedacted(flaggedStartResult.output);

						const flaggedSessionId = extractSessionId(
							flaggedStartResult.output,
						);
						createdSessionIds.add(flaggedSessionId);

						const flaggedSession = await getSessionById(
							flaggedSessionId,
							harness.apiKey!,
						);
						const flaggedStealthConfig = asRecord(
							flaggedSession['stealthConfig'],
						);

						expect(flaggedSession['proxySource']).toBe('external');
						expect(flaggedStealthConfig?.['humanizeInteractions']).toBe(true);
						expect(flaggedStealthConfig?.['autoCaptchaSolving']).toBe(true);
						expect(flaggedSession['solveCaptcha']).toBe(true);
					} finally {
						const stopResult = runBrowserCommand(
							['stop'],
							environment,
							projectRoot,
						);
						if (stopResult.status !== 0) {
							console.warn(
								`Cleanup warning: browser stop failed with status ${stopResult.status}.`,
							);
						}

						for (const sessionId of createdSessionIds) {
							try {
								await releaseSessionById(sessionId, harness.apiKey!);
							} catch {
								// Best effort cleanup.
							}
						}
					}
				},
			);
		},
		90_000,
	);
});
