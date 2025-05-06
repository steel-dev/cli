import {useEffect, useState} from 'react';

export function useTask(): [
	string,
	string | null,
	boolean,
	string | null,
	React.Dispatch<React.SetStateAction<string | null>>,
	React.Dispatch<React.SetStateAction<boolean>>,
	React.Dispatch<React.SetStateAction<string | null>>,
] {
	const [task, setTask] = useState<string | null>(null);
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | null>(null);
	const [state, setState] = useState<string>('pending');

	useEffect(() => {
		setState(
			task ? 'success' : loading ? 'loading' : error ? 'error' : 'pending',
		);
	}, [task, loading, error]);

	return [state, task, loading, error, setTask, setLoading, setError];
}
