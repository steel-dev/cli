import {useEffect} from 'react';

export const useFullscreen = () => {
	useEffect(() => {
		// Clear screen and hide cursor
		process.stdout.write('\x1b[2J\x1b[0f\x1b[?25l');

		return () => {
			// Restore cursor on exit
			process.stdout.write('\x1b[?25h\n');
		};
	}, []);
};
