import {useEffect, useState} from 'react';
import type {LoadingState, OtherStates} from '../utils/types.js';

export function useTask(): [
	LoadingState | OtherStates | undefined,
	string | object | boolean | undefined,
	boolean,
	string | undefined,
	React.Dispatch<React.SetStateAction<string | object | boolean | undefined>>,
	React.Dispatch<React.SetStateAction<boolean>>,
	React.Dispatch<React.SetStateAction<string | undefined>>,
] {
	const [task, setTask] = useState();
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | undefined>();
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
