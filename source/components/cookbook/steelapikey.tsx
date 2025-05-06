import React, {useEffect} from 'react';
import {Text} from 'ink';
import {getApiKey} from '../../utils/session.js';
import {Task} from 'ink-task-list';
import {useTask} from '../../hooks/usetask.js';

export default function SteelApiKey() {
	const [state, task, loading, error, setTask, setLoading, setError] =
		useTask();

	useEffect(() => {
		const fetchApiKey = async () => {
			try {
				setLoading(true);
				const apiKey = await getApiKey();
				setTask(apiKey);
				setLoading(false);
			} catch (error) {
				console.error('Error fetching API key:', error);
				setError('Error fetching API key');
				setLoading(false);
			}
		};

		fetchApiKey();
	}, []);

	return (
		<Task label="Grabbing Steel API Key" state={state}>
			<Text>{task ? `API Key: ${task?.apiKey}` : 'Loading API Key...'}</Text>
		</Task>
	);
}
