import React, {useEffect, useState} from 'react';
import {spawn} from 'child_process';
import {CONFIG_DIR, REPO_URL} from '../../utils/constants.js';
import path from 'path';
import zod from 'zod';
import {option} from 'pastel';
import Spinner from 'ink-spinner';
import {Text} from 'ink';
import Callout from '../../components/callout.js';

export const description = 'Stops any running dev instance of Steel Browser';

export const options = zod.object({
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
	force: zod
		.boolean()
		.describe(
			option({
				description: 'Force stop containers and remove orphaned containers',
				alias: 'f',
			}),
		)
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

export default function Stop({options}: Props) {
	const [loading, setLoading] = useState(false);
	const [dockerError, setDockerError] = useState(false);
	const [status, setStatus] = useState('');
	const [output, setOutput] = useState<string>('');
	const [stopped, setStopped] = useState(false);

	useEffect(() => {
		async function stop() {
			setLoading(true);
			setStatus('Checking Docker status...');

			const dockerRunning = await isDockerRunning();
			if (!dockerRunning) {
				setDockerError(true);
				setLoading(false);
				return;
			}

			setStatus(
				options?.force
					? 'Force stopping Docker Compose...'
					: 'Stopping Docker Compose...',
			);

			const folderName = path.basename(REPO_URL, '.git');

			// Build docker-compose command arguments
			const dockerCommand = 'docker-compose';
			let dockerArgs: string[];

			if (options?.force) {
				// Use kill for force stop (sends SIGKILL)
				dockerArgs = ['-f', 'docker-compose.dev.yml', 'kill'];
			} else {
				// Use down for graceful stop
				dockerArgs = ['-f', 'docker-compose.dev.yml', 'down'];
			}

			const dockerCompose = spawn(dockerCommand, dockerArgs, {
				cwd: path.join(CONFIG_DIR, folderName),
				stdio: ['pipe', 'pipe', 'pipe'], // Capture stdout and stderr
				env: {
					...process.env,
					ENABLE_VERBOSE_LOGGING: options?.verbose || 'false',
				},
			});

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

				// If we used kill (force), we still need to clean up with down
				if (options?.force) {
					setStatus('Cleaning up containers...');
					const cleanup = spawn(
						'docker-compose',
						['-f', 'docker-compose.dev.yml', 'down'],
						{
							cwd: path.join(CONFIG_DIR, folderName),
							env: {
								...process.env,
								ENABLE_VERBOSE_LOGGING: options?.verbose || 'false',
							},
						},
					);

					cleanup.on('close', () => {
						setStatus('Development environment force stopped and cleaned up');
						setStopped(true);
						setLoading(false);
					});

					cleanup.on('error', () => {
						setStatus('Development environment force stopped (cleanup failed)');
						setStopped(true);
						setLoading(false);
					});
				} else {
					setStatus('Development environment stopped successfully');
					setStopped(true);
					setLoading(false);
				}
			});

			dockerCompose.on('error', error => {
				console.error('Error stopping Docker Compose:', error);
				setDockerError(true);
				setLoading(false);
			});
		}
		stop();
	}, []);

	if (dockerError) {
		return (
			<Callout variant="failed" title="Stop Failed">
				Docker is not running or there was an error stopping the containers.
			</Callout>
		);
	}

	if (loading) {
		return (
			<Callout variant="info" title="Stopping Development Environment">
				<Text>
					<Spinner type="dots" /> {status + '\n'}
				</Text>
				{output && (
					<Text dimColor>{output.split('\n').slice(-5).join('\n')}</Text>
				)}
			</Callout>
		);
	}

	if (stopped) {
		return (
			<Callout variant="success" title="Development Environment Stopped">
				{options?.force
					? 'All containers have been force stopped and cleaned up successfully.'
					: 'All containers have been stopped successfully.'}
			</Callout>
		);
	}

	return (
		<Callout variant="info" title="Stopping Development Environment">
			<Text>
				<Spinner type="dots" /> {status}
			</Text>
		</Callout>
	);
}
