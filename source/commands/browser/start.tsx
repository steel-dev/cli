import React, {useEffect, useState} from 'react';
import {spawn} from 'child_process';
import open from 'open';
import {CONFIG_DIR, REPO_URL} from '../../utils/constants.js';
import path from 'path';
import zod from 'zod';
import {option} from 'pastel';
import Spinner from 'ink-spinner';
import {Text} from 'ink';
import Callout from '../../components/callout.js';

export const description = 'Starts the development environment';

export const options = zod.object({
	port: zod
		.number()
		.describe(
			option({
				description: 'Port number',
				alias: 'p',
			}),
		)
		.default(3000)
		.optional(),
	verbose: zod
		.string()
		.describe(
			option({
				description: 'Enable verbose logging',
				alias: 'v',
			}),
		)
		.optional(),
	docker_check: zod
		.string()
		.describe(option({description: 'Verify Docker is running', alias: 'dc'}))
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

function isDockerRunning(): Promise<boolean> {
	return new Promise(resolve => {
		const isRunning = spawn('docker', ['info']);
		isRunning.on('close', code => {
			resolve(code === 0);
		});
		isRunning.on('error', () => {
			resolve(false);
		});
	});
}

async function waitForApiHealth(
	port: number,
	maxRetries = 30,
	retryDelay = 500,
): Promise<boolean> {
	const healthUrl = `http://localhost:${port}/v1/health`;

	for (let attempt = 1; attempt <= maxRetries; attempt++) {
		try {
			const response = await fetch(healthUrl);
			if (response.ok) {
				return true;
			}
		} catch {
			// API not ready yet, continue polling
		}

		// Wait before next attempt
		await new Promise(resolve => setTimeout(resolve, retryDelay));
	}

	return false;
}

export default function Start({options}: Props) {
	const [loading, setLoading] = useState(false);
	const [dockerError, setDockerError] = useState(false);
	const [status, setStatus] = useState('');
	const [output, setOutput] = useState<string>('');
	const [apiReady, setApiReady] = useState(false);

	useEffect(() => {
		async function start() {
			const port = options?.port || 3000;
			setLoading(true);
			setStatus('Cloning repository...');

			spawn('git', ['clone', REPO_URL], {
				cwd: CONFIG_DIR,
			});

			const dockerRunning = await isDockerRunning();
			if (!dockerRunning) {
				setDockerError(true);
				setLoading(false);
				return;
			}

			setStatus('Starting Docker Compose...');

			const folderName = path.basename(REPO_URL, '.git');

			const dockerCompose = spawn(
				'docker-compose',
				['-f', 'docker-compose.dev.yml', 'up', '-d'],
				{
					cwd: path.join(CONFIG_DIR, folderName),
					stdio: ['pipe', 'pipe', 'pipe'], // Capture stdout and stderr
					env: {
						...process.env,
						API_PORT: String(port),
						ENABLE_VERBOSE_LOGGING: options?.verbose || 'false',
					},
				},
			);

			// Stream stdout to output state
			dockerCompose.stdout?.on('data', data => {
				const text = data.toString();
				setOutput(prev => prev + text);
			});

			// Stream stderr to output state
			dockerCompose.stderr?.on('data', data => {
				const text = data.toString();
				setOutput(prev => prev + text);
			});

			dockerCompose.on('close', async code => {
				if (code !== 0) {
					setDockerError(true);
					setLoading(false);
					return;
				}

				setStatus('Waiting for API to be ready...');
				setOutput(''); // Clear docker output for cleaner display

				const isApiHealthy = await waitForApiHealth(port);
				if (!isApiHealthy) {
					setDockerError(true);
					setStatus('API failed to start within expected time');
					setLoading(false);
					return;
				}

				setStatus('Opening browser...');
				setApiReady(true);
				setLoading(false);
				await open('http://localhost:5173');
			});

			dockerCompose.on('error', error => {
				console.error('Error starting Docker Compose:', error);
				setDockerError(true);
				setLoading(false);
			});
		}
		start();
	}, []);

	if (dockerError) {
		return (
			<Callout variant="failed" title="Startup Failed">
				{status.includes('API failed')
					? status
					: 'Docker is not running. Please start Docker and try again.'}
			</Callout>
		);
	}

	if (loading) {
		return (
			<Callout variant="info" title="Starting Development Environment">
				<Text>
					<Spinner type="dots" /> {status + '\n'}
				</Text>
				{output && (
					<Text dimColor>{output.split('\n').slice(-5).join('\n')}</Text>
				)}
			</Callout>
		);
	}

	if (apiReady) {
		return (
			<Callout variant="success" title="Development Environment Ready">
				Browser opened at http://localhost:5173
			</Callout>
		);
	}

	return (
		<Callout variant="info" title="Starting Development Environment">
			<Text>
				<Spinner type="dots" /> {status}
			</Text>
		</Callout>
	);
}
