#!/usr/bin/env node

import {useEffect} from 'react';
import open from 'open';

export const description = 'Navigates to Steel Browser Repository';

export default function Docs() {
	useEffect(() => {
		async function start() {
			console.log('🛠️  Opening Repository...');
			console.log('Hey, while you are there, mind dropping a star? ⭐ 👀');
			await open('https://github.com/steel-dev/steel-browser');
		}
		start();
	}, []);
}
