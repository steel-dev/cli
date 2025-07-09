import {useEffect} from 'react';
// import {spawn} from 'child_process';
import open from 'open';
// import {CONFIG_DIR, REPO_URL} from '../utils/constants.js';
// import path from 'path';
// import Spinner from 'ink-spinner';
// import {Text} from 'ink';

export const description = 'Navigates to Steel Browser Repository';

export default function Docs() {
	useEffect(() => {
		async function start() {
			console.log('🛠️  Opening Repository...');
			await open('https://github.com/steel-dev/steel-browser');
			console.log('Hey while you are there, mind dropping a star? ⭐');
		}
		start();
	}, []);
}
