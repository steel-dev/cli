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

function isDockerRunning() {
	try {
		spawn('docker', ['info']);
		return true;
	} catch {
		return false;
	}
}

export default function Start({options}: Props) {
	const [loading, setLoading] = useState(false);
	const [dockerError, setDockerError] = useState(false);
	const [status, setStatus] = useState('');

	useEffect(() => {
		async function start() {
			setLoading(true);
			setStatus('Cloning repository...');

			spawn('git', ['clone', REPO_URL], {
				cwd: CONFIG_DIR,
			});

			if (options?.docker_check && !isDockerRunning()) {
				setDockerError(true);
				setLoading(false);
				return;
			}

			setStatus('Starting Docker Compose...');
			setLoading(false);

			const folderName = path.basename(REPO_URL, '.git');

			spawn('docker-compose', ['-f', 'docker-compose.dev.yml', 'up', '-d'], {
				cwd: path.join(CONFIG_DIR, folderName),
				stdio: 'inherit',
				env: {
					...process.env,
					API_PORT: String(options?.port || 3000),
					ENABLE_VERBOSE_LOGGING: options?.verbose || 'false',
				},
			});

			setStatus('Opening browser...');
			await open('http://localhost:5173');
		}
		start();
	}, []);

	if (dockerError) {
		return (
			<Callout variant="failed" title="Docker Not Running">
				Docker is not running. Please start Docker and try again.
			</Callout>
		);
	}

	if (loading) {
		return (
			<Callout variant="info" title="Starting Development Environment">
				<Text>
					<Spinner type="dots" /> {status}
				</Text>
			</Callout>
		);
	}

	return (
		<Callout variant="success" title="Development Environment Started">
			Browser opened at http://localhost:5173
		</Callout>
	);
}
