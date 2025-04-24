import React, {useEffect, useState} from 'react';
import {writeFile} from 'fs/promises';
import {exec} from 'child_process';
import {CONFIG_DIR} from '../utils/constants.js';
import path from 'path';
import fs from 'fs';
import Spinner from 'ink-spinner';
import {Text} from 'ink';

const composeFileUrl =
	'https://raw.githubusercontent.com/steel-dev/steel-browser/main/docker-compose.yml';

async function downloadFile(url: string, dest: string) {
	console.log(`⬇️ Downloading ${url}`);
	const res = await fetch(url);
	if (!res.ok) {
		throw new Error(`Failed to download file: ${res.status} ${res.statusText}`);
	}

	const content = await res.text();
	await writeFile(dest, content);
	console.log(`✅ Saved to ${dest}`);
}

export default function Start() {
	const [loading, setLoading] = useState(false);
	useEffect(() => {
		async function start() {
			setLoading(true);
			if (!fs.existsSync(path.join(CONFIG_DIR, 'docker-compose.yml'))) {
				console.log('⬇️ Downloading docker-compose.yml...');
				await downloadFile(
					composeFileUrl,
					path.join(CONFIG_DIR, 'docker-compose.yml'),
				);
			}

			setLoading(false);
			console.log('🚀 Starting Docker Compose...');
			exec('docker-compose up');
		}
		start();
	}, []);

	return <Text>{loading ? <Spinner type="dots" /> : null}</Text>;
}
