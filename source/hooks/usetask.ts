import {useEffect, useState} from 'react';
import type {LoadingState, OtherStates} from '../utils/types.js';

export function useTask(): [
	LoadingState | OtherStates | undefined,
	any,
	boolean,
	string | null,
	React.Dispatch<React.SetStateAction<any>>,
	React.Dispatch<React.SetStateAction<boolean>>,
	React.Dispatch<React.SetStateAction<string | null>>,
] {
	const [task, setTask] = useState();
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | null>(null);
	const [state, setState] = useState<LoadingState | OtherStates | undefined>(
		'pending',
	);

	useEffect(() => {
		// console.log('Task:', task);
		// console.log('Loading:', loading);
		// console.log('Error:', error);
		setState(
			loading ? 'loading' : task ? 'success' : error ? 'error' : 'pending',
		);
	}, [loading, task, error]);

	return [state, task, loading, error, setTask, setLoading, setError];
}
