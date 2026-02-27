import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import {
	type Mock,
	afterEach,
	beforeEach,
	describe,
	expect,
	test,
	vi,
} from 'vitest';
import {
	getScrapeOutputText,
	getHostedAssetUrl,
	parseScrapeFormatOption,
	requestTopLevelApi,
	resolveTopLevelToolUrl,
} from '../../source/utils/topLevelTools';

function createJsonResponse(status: number, payload: unknown): Response {
	return {
		ok: status >= 200 && status < 300,
		status,
		statusText: status >= 200 && status < 300 ? 'OK' : 'Error',
		text: async () => JSON.stringify(payload),
	} as Response;
}

function createTempConfigDirectory(): string {
	return fs.mkdtempSync(path.join(os.tmpdir(), 'steel-top-level-tools-test-'));
}

const originalFetch = globalThis.fetch;
let fetchMock: Mock<typeof fetch>;

beforeEach(() => {
	fetchMock = vi.fn() as Mock<typeof fetch>;
	globalThis.fetch = fetchMock;
});

afterEach(() => {
	globalThis.fetch = originalFetch;
});

describe('top-level tool argument parsing', () => {
	test('parses scrape format option from comma-separated values', () => {
		expect(parseScrapeFormatOption('html, markdown')).toEqual([
			'html',
			'markdown',
		]);
	});

	test('throws for invalid scrape format value', () => {
		expect(() => parseScrapeFormatOption('html,xml')).toThrow(
			'Invalid scrape format value(s): xml.',
		);
	});

	test('resolves URL from flag first and validates it', () => {
		expect(
			resolveTopLevelToolUrl('https://steel.dev/docs', 'https://example.com'),
		).toBe('https://steel.dev/docs');
		expect(resolveTopLevelToolUrl(undefined, 'hackernews.com')).toBe(
			'https://hackernews.com',
		);
		expect(resolveTopLevelToolUrl(undefined, 'localhost:3000')).toBe(
			'https://localhost:3000',
		);
		expect(() => resolveTopLevelToolUrl(undefined, 'not-a-url')).toThrow(
			'Invalid URL: not-a-url',
		);
	});

	test('extracts hosted asset URL from response payload', () => {
		expect(getHostedAssetUrl({url: 'https://files.steel.dev/asset.png'})).toBe(
			'https://files.steel.dev/asset.png',
		);
		expect(() => getHostedAssetUrl({})).toThrow(
			'Steel API response did not include a URL.',
		);
	});

	test('prefers markdown scrape output by default', () => {
		expect(
			getScrapeOutputText({
				content: {
					markdown: '# Headline',
					html: '<h1>Headline</h1>',
				},
			}),
		).toBe('# Headline');
	});

	test('uses requested scrape format ordering when available', () => {
		expect(
			getScrapeOutputText(
				{
					content: {
						markdown: '# Headline',
						html: '<h1>Headline</h1>',
					},
				},
				['html'],
			),
		).toBe('<h1>Headline</h1>');
	});

	test('stringifies readability content when no text format exists', () => {
		expect(
			getScrapeOutputText({
				content: {
					readability: {
						title: 'Headline',
					},
				},
			}),
		).toBe(`{
  "title": "Headline"
}`);
	});
});

describe('top-level tool API contract', () => {
	test('calls cloud endpoint with auth header and JSON payload', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock.mockResolvedValueOnce(
				createJsonResponse(200, {url: 'https://files.steel.dev/pdf-file'}),
			);

			const response = await requestTopLevelApi<{url: string}>(
				'/pdf',
				{
					url: 'https://steel.dev',
					delay: 1500,
				},
				{
					environment: {
						STEEL_API_KEY: 'env-api-key',
						STEEL_CONFIG_DIR: configDirectory,
					},
				},
			);

			expect(response).toEqual({url: 'https://files.steel.dev/pdf-file'});
			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[0]).toBe('https://api.steel.dev/v1/pdf');
			expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Steel-Api-Key': 'env-api-key',
				},
				body: JSON.stringify({
					url: 'https://steel.dev',
					delay: 1500,
				}),
			});
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('uses API key from config when environment key is absent', async () => {
		const configDirectory = createTempConfigDirectory();
		const configPath = path.join(configDirectory, 'config.json');

		try {
			fs.mkdirSync(configDirectory, {recursive: true});
			fs.writeFileSync(
				configPath,
				JSON.stringify(
					{
						apiKey: 'config-api-key',
					},
					null,
					2,
				),
				'utf-8',
			);

			fetchMock.mockResolvedValueOnce(
				createJsonResponse(200, {url: 'https://files.steel.dev/screenshot'}),
			);

			await requestTopLevelApi(
				'/screenshot',
				{
					url: 'https://steel.dev',
				},
				{
					environment: {
						STEEL_CONFIG_DIR: configDirectory,
					},
				},
			);

			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
				headers: {
					'Content-Type': 'application/json',
					'Steel-Api-Key': 'config-api-key',
				},
			});
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('calls local endpoint when explicit api-url is provided', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock.mockResolvedValueOnce(createJsonResponse(200, {url: 'ok'}));

			await requestTopLevelApi(
				'/screenshot',
				{
					url: 'https://steel.dev',
					fullPage: true,
				},
				{
					apiUrl: 'https://steel.self-hosted.dev/v1/',
					environment: {
						STEEL_CONFIG_DIR: configDirectory,
					},
				},
			);

			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[0]).toBe(
				'https://steel.self-hosted.dev/v1/screenshot',
			);
			expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
				headers: {
					'Content-Type': 'application/json',
				},
			});
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('uses browser.apiUrl from config in local mode', async () => {
		const configDirectory = createTempConfigDirectory();
		const configPath = path.join(configDirectory, 'config.json');

		try {
			fs.mkdirSync(configDirectory, {recursive: true});
			fs.writeFileSync(
				configPath,
				JSON.stringify(
					{
						browser: {
							apiUrl: 'https://configured.local/v1/',
						},
					},
					null,
					2,
				),
				'utf-8',
			);

			fetchMock.mockResolvedValueOnce(createJsonResponse(200, {url: 'ok'}));

			await requestTopLevelApi(
				'/pdf',
				{
					url: 'https://steel.dev',
				},
				{
					local: true,
					environment: {
						STEEL_CONFIG_DIR: configDirectory,
					},
				},
			);

			expect(fetchMock).toHaveBeenCalledTimes(1);
			expect(fetchMock.mock.calls[0]?.[0]).toBe(
				'https://configured.local/v1/pdf',
			);
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('throws missing auth error in cloud mode', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			await expect(
				requestTopLevelApi(
					'/scrape',
					{
						url: 'https://steel.dev',
					},
					{
						environment: {
							STEEL_CONFIG_DIR: configDirectory,
						},
					},
				),
			).rejects.toMatchObject({
				code: 'MISSING_AUTH',
			});
			expect(fetchMock).not.toHaveBeenCalled();
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});

	test('surfaces API error details from non-2xx responses', async () => {
		const configDirectory = createTempConfigDirectory();

		try {
			fetchMock.mockResolvedValueOnce(
				createJsonResponse(422, {message: 'Invalid URL'}),
			);

			await expect(
				requestTopLevelApi(
					'/scrape',
					{
						url: 'https://steel.dev',
					},
					{
						environment: {
							STEEL_API_KEY: 'env-api-key',
							STEEL_CONFIG_DIR: configDirectory,
						},
					},
				),
			).rejects.toThrow('Steel API request failed (422): Invalid URL');
		} finally {
			fs.rmSync(configDirectory, {recursive: true, force: true});
		}
	});
});
