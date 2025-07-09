import {useEffect} from 'react';
// import {spawn} from 'child_process';
import open from 'open';
// import {CONFIG_DIR, REPO_URL} from '../utils/constants.js';
// import path from 'path';
// import Spinner from 'ink-spinner';
// import {Text} from 'ink';

export const description = 'Navigates to Steel Discord Server';

export default function Support() {
	useEffect(() => {
		async function start() {
			console.log('🧑‍💻  Opening Discord...');
			await open('https://discord.com/invite/steel-dev');
		}
		start();
	}, []);
}
