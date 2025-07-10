import {Box, Text, useInput, Newline} from 'ink';
import {useEffect, useState} from 'react';
import TextInput from 'ink-text-input';
import ProgressBar from './progressbar.js';
// import {useFullscreen} from '../hooks/usefullscreen.js';
import Spinner from 'ink-spinner';

type Props = {
	data: any[];
	loading: boolean;
	error: Error | null;
	method: string;
	endpoint: string;
};

export default function GetDashboard({
	data,
	loading,
	error,
	method,
	endpoint,
}: Props) {
	const [index, setIndex] = useState(0);
	const [search, setSearch] = useState('');
	// const [loading, data] = useApi({method, endpoint, resultObject});
	const [filteredData, setFilteredData] = useState<any[]>(data);

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
		setIndex(0);
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

	// useFullscreen();

	if (loading) {
		return (
			<Box justifyContent="center" alignItems="center">
				<Text>
					<Spinner type="pong" />
				</Text>
			</Box>
		);
	} else if (error) {
		return (
			<Box
				// borderColor="red"
				borderStyle="bold"
				flexDirection="row"
				display="flex"
				justifyContent="space-around"
			>
				<Text color="red">{error.message}</Text>
			</Box>
		);
	}

	return (
		<Box>
			{data.length > 0 ? (
				<Box flexDirection="column">
					<Box
						marginBottom={1}
						// borderColor="red"
						borderStyle="bold"
					>
						<Box marginLeft={1}>
							<Text color="cyan">Search: </Text>
						</Box>
						<TextInput value={search} onChange={setSearch} />
					</Box>
					<Text color="green" bold>
						{method}{' '}
						<Text color="blue" bold>
							/{endpoint}
						</Text>
					</Text>

					<Box
						// borderColor="red"
						borderStyle="bold"
						flexDirection="column"
					>
						<Box marginLeft={1} flexDirection="column">
							{current ? (
								<Box flexDirection="column">
									<Text>{`{`}</Text>
									{keys.map(key => (
										<Text key={key}>
											{'  '}
											<Text color="yellow">{key}:</Text>{' '}
											<Text
												color={
													typeof current[key] === 'number' ? 'green' : 'blue'
												}
											>
												{current[key]?.toString()}
											</Text>
										</Text>
									))}
									<Text>{`}`}</Text>
								</Box>
							) : (
								<Text color="red">No results</Text>
							)}
						</Box>
					</Box>
					<Newline />
					<Box
						// borderColor="red"
						borderStyle="bold"
						flexDirection="row"
						display="flex"
						justifyContent="space-around"
					>
						<Text>
							{' '}
							{index + 1 > filteredData.length
								? filteredData.length
								: index + 1}
							/{filteredData.length}
						</Text>
						<ProgressBar
							percent={
								filteredData.length > 0 ? (index + 1) / filteredData.length : 0
							}
							left="["
							right="]"
							character="="
						/>
					</Box>
				</Box>
			) : (
				<Text>No sessions found :(</Text>
			)}
		</Box>
	);
}
