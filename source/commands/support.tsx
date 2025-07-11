import {useEffect} from 'react';
import open from 'open';

export const description = 'Navigates to Steel Discord Server';

export default function Support() {
	useEffect(() => {
		async function start() {
			console.log('💁 Opening Discord...');
			await open('https://discord.com/invite/steel-dev');
		}
		start();
	}, []);
}
