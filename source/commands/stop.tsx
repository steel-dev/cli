import path from 'path';
import {useEffect} from 'react';
import {exec} from 'child_process';
import {CONFIG_DIR, REPO_URL} from '../utils/constants.js';

export const description = 'Stops the development server';

export default function Stop() {
	const folderName = path.basename(REPO_URL, '.git');
	useEffect(() => {
		async function stop() {
			console.log('🚀 Stopping Docker Compose...');
			exec('docker-compose down', {
				cwd: path.join(CONFIG_DIR, folderName),
			});
		}
		stop();
	}, []);
}
