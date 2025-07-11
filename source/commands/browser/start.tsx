import {useEffect, useState} from 'react';
import {spawn} from 'child_process';
import open from 'open';
import {CONFIG_DIR, REPO_URL} from '../../utils/constants.js';
import path from 'path';
import zod from 'zod';
import {option} from 'pastel';
import Spinner from 'ink-spinner';
import {Text} from 'ink';

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
	} catch (error) {
		return false;
	}
}

export default function Start({options}: Props) {
	const [loading, setLoading] = useState(false);
	useEffect(() => {
		async function start() {
			setLoading(true);

			spawn('git', ['clone', REPO_URL], {
				cwd: CONFIG_DIR,
			});

			if (options?.docker_check && !isDockerRunning()) {
				console.log('⚠️ Docker is not running. Please start it and try again.');
				return;
			}

			setLoading(false);

			console.log('🚀 Starting Docker Compose...');

			const folderName = path.basename(REPO_URL, '.git');

			spawn('docker-compose', ['-f', 'docker-compose.dev.yml', 'up', '-d'], {
				cwd: path.join(CONFIG_DIR, folderName),
				stdio: 'inherit',
			});

			console.log('🖥️  Opening Browser...');
			await open('http://localhost:5173');
		}
		start();
	}, []);

	return <Text>{loading ? <Spinner type="dots" /> : null}</Text>;
}
