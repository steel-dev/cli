import {Box, Text, useInput, Newline} from 'ink';
import React, {useEffect, useState} from 'react';
import {API_PATH} from '../../utils/config.js';
import {getApiKey} from '../../utils/session.js';
import TextInput from 'ink-text-input';
// @ts-ignore
// import ProgressBar from 'ink-progress-bar';
import Spinner from 'ink-spinner';

export default function getSessions() {
	const [data, setData] = useState<any[]>([]);
	const [loading, setLoading] = useState(true);
	const [index, setIndex] = useState(0);
	const [search, setSearch] = useState('');
	const [filteredData, setFilteredData] = useState<any[]>(data);

	useEffect(() => {
		async function fetchData() {
			try {
				const apiKey = await getApiKey();
				if (!apiKey || !apiKey.apiKey || !apiKey.name) {
					throw new Error('API key not found');
				}
				const response = await fetch(`${API_PATH}/sessions`, {
					method: 'GET',
					headers: {
						'Content-Type': 'application/json',
						'Steel-Api-Key': apiKey?.apiKey,
					},
				});
				const data = await response.json();
				setData(data.sessions);
			} catch (error) {
				console.error('Error fetching sessions:', error);
			}
			setLoading(false);
		}
		fetchData();
	}, []);

	useEffect(() => {
		if (search === '') {
			setFilteredData(data);
		} else {
			const lower = search.toLowerCase();
			setFilteredData(
				data.filter(entry =>
					Object.values(entry).some(val =>
						val?.toString().toLowerCase().includes(lower),
					),
				),
			);
		}
		setIndex(0); // Reset index on search change
	}, [search, data]);

	useInput((_, key) => {
		if (key.leftArrow) {
			setIndex(i => (i > 0 ? i - 1 : i));
		} else if (key.rightArrow) {
			setIndex(i => (i < filteredData.length - 1 ? i + 1 : i));
		}
	});

	const current = filteredData[index];
	const keys = current ? Object.keys(current) : [];

	const useFullscreen = () => {
		useEffect(() => {
			// Clear screen and hide cursor
			process.stdout.write('\x1b[2J\x1b[0f\x1b[?25l');

			return () => {
				// Restore cursor on exit
				process.stdout.write('\x1b[?25h\n');
			};
		}, []);
	};

	useFullscreen();

	return (
		<Box>
			{loading ? (
				<Text color="yellow">
					<Spinner type="dots" />
				</Text>
			) : data.length > 0 ? (
				<Box flexDirection="column">
					<Box marginBottom={1} borderColor="red" borderStyle="bold">
						<Box marginLeft={1}>
							<Text color="cyan">Search: </Text>
						</Box>
						<TextInput value={search} onChange={setSearch} />
					</Box>
					<Box borderColor="red" borderStyle="bold" flexDirection="column">
						<Box marginLeft={1} flexDirection="column">
							{current ? (
								keys.map(key => (
									<Text key={key}>
										<Text color="yellow">{key}:</Text>{' '}
										<Text
											color={
												typeof current[key] === 'number' ? 'green' : 'blue'
											}
										>
											{current[key]?.toString()}
										</Text>
									</Text>
								))
							) : (
								<Text color="red">No results</Text>
							)}
						</Box>
					</Box>

					<Newline />
					<Box>
						{/* <ProgressBar
							percent={
								filteredData.length > 0 ? (index + 1) / filteredData.length : 0
							}
							left="["
							right="]"
							character="="
						/> */}
						<Box borderColor="red" borderStyle="bold" flexDirection="column">
							<Text>
								{' '}
								{index + 1}/{filteredData.length}
							</Text>
						</Box>
					</Box>
				</Box>
			) : (
				<Text>No sessions found :(</Text>
			)}
		</Box>
	);
}
