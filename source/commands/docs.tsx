import {useEffect} from 'react';
import open from 'open';

export const description = 'Navigates to Steel Docs';

export default function Docs() {
	useEffect(() => {
		async function start() {
			console.log('📝 Opening Docs...');
			await open('https://docs.steel.dev/overview/intro-to-steel');
		}
		start();
	}, []);
}
