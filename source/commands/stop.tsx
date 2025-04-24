import {useEffect} from 'react';
import {exec} from 'child_process';

export default function Stop() {
	useEffect(() => {
		async function stop() {
			console.log('🚀 Stopping Docker Compose...');
			exec('docker-compose down');
		}

		stop();
	}, []);
}
