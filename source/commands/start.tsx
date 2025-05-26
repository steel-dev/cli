import React, {useEffect, useState} from 'react';
import {spawn} from 'child_process';
import open from 'open';
import {CONFIG_DIR, REPO_URL} from '../utils/constants.js';
import path from 'path';
import Spinner from 'ink-spinner';
import {Text} from 'ink';

function isDockerRunning() {
	try {
		spawn('docker', ['info']);
		return true;
	} catch (error) {
		return false;
	}
}

export default function Start() {
	const [loading, setLoading] = useState(false);
	useEffect(() => {
		async function start() {
			setLoading(true);

			spawn('git', ['clone', REPO_URL], {
				cwd: CONFIG_DIR,
			});

			if (!isDockerRunning()) {
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
