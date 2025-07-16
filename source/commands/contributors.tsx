// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
import React, {useState, useEffect} from 'react';
import {Box, Text, useApp, useInput} from 'ink';
import {Octokit} from 'octokit';
// import Image from 'ink-image';
import Spinner from 'ink-spinner';
import figures from 'figures';

// Define types
interface Contributor {
	login?: string;
	id?: number;
	avatar_url: string;
	contributions: number;
	name: string | null;
	bio: string | null;
	additions?: number;
	deletions?: number;
	commits?: number;
}

export const description = 'Show project contributors from GitHub';

export default function Contributors() {
	const [contributors, setContributors] = useState<Contributor[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [currentPage, setCurrentPage] = useState(0);
	const [selectedCardIndex, setSelectedCardIndex] = useState(0);
	const contributorsPerPage = 3;
	const {exit} = useApp();

	// Handle keyboard input for carousel navigation
	useInput((input, key) => {
		if (input === 'q' || key.escape) {
			exit();
		} else if (key.rightArrow) {
			// If holding shift, move between cards on the same page
			if (key.shift) {
				setSelectedCardIndex(prev => {
					const maxIndex = Math.min(
						contributorsPerPage - 1,
						contributors.length - currentPage * contributorsPerPage - 1,
					);
					return prev < maxIndex ? prev + 1 : 0;
				});
			} else {
				// Move to next page
				const maxPages = Math.ceil(contributors.length / contributorsPerPage);
				setCurrentPage(prev => (prev + 1) % maxPages);
				setSelectedCardIndex(0);
			}
		} else if (key.leftArrow) {
			// If holding shift, move between cards on the same page
			if (key.shift) {
				setSelectedCardIndex(prev => {
					const maxIndex = Math.min(
						contributorsPerPage - 1,
						contributors.length - currentPage * contributorsPerPage - 1,
					);
					return prev > 0 ? prev - 1 : maxIndex;
				});
			} else {
				// Move to previous page
				const maxPages = Math.ceil(contributors.length / contributorsPerPage);
				setCurrentPage(prev => (prev - 1 + maxPages) % maxPages);
				setSelectedCardIndex(0);
			}
		}
	});

	// Auto rotate carousel every 5 seconds
	useEffect(() => {
		if (!loading && contributors.length > 0) {
			const maxPages = Math.ceil(contributors.length / contributorsPerPage);
			const timer = setInterval(() => {
				setCurrentPage(prev => (prev + 1) % maxPages);
			}, 8000);

			return () => clearInterval(timer);
		}
		return undefined;
	}, [loading, contributors.length]);

	// Fetch contributors data
	useEffect(() => {
		const fetchContributors = async (): Promise<void> => {
			try {
				const octokit = new Octokit();

				// Get basic contributor information
				const {data: contribData} = await octokit.request(
					'GET /repos/{owner}/{repo}/contributors',
					{
						owner: 'steel-dev',
						repo: 'steel-browser',
						per_page: 25,
						headers: {
							accept: 'application/vnd.github.v3+json',
						},
					},
				);

				// Get commit stats for each contributor
				const contributorsWithDetails = await Promise.all(
					contribData.map(async contributor => {
						// Get additional user details
						const {data: userData} = await octokit.request(
							'GET /users/{username}',
							{
								username: contributor.login,
							},
						);

						// Get commit stats
						const {data: commitStats} = await octokit.request(
							'GET /repos/{owner}/{repo}/stats/contributors',
							{
								owner: 'steel-dev',
								repo: 'steel-browser',
							},
						);
						console.log(commitStats);

						const stats = commitStats.find(
							stat => stat.author.login === contributor.login,
						);
						let additions = 0;
						let deletions = 0;
						let commits = 0;

						if (stats) {
							stats.weeks.forEach(week => {
								additions += week.a;
								deletions += week.d;
								commits += week.c;
							});
						}

						return {
							...contributor,
							name: userData.name,
							bio: userData.bio,
							additions,
							deletions,
							commits,
						};
					}),
				);

				setContributors(contributorsWithDetails);
				setLoading(false);
			} catch (err) {
				setError(`Failed to fetch contributors: ${(err as Error).message}`);
				setLoading(false);
			}
		};

		fetchContributors();
	}, []);

	// Display loading spinner while fetching data
	if (loading) {
		return (
			<Box padding={1}>
				<Text>
					<Text color="green">
						<Spinner type="dots" />
					</Text>{' '}
					Loading contributors...
				</Text>
			</Box>
		);
	}

	// Display error if any
	if (error) {
		return (
			<Box padding={1}>
				<Text color="red">Error: {error}</Text>
			</Box>
		);
	}

	// Display empty message if no contributors found
	if (contributors.length === 0) {
		return (
			<Box padding={1}>
				<Text>No contributors found.</Text>
			</Box>
		);
	}

	// Get current contributors to display
	const maxPages = Math.ceil(contributors.length / contributorsPerPage);
	const startIndex = currentPage * contributorsPerPage;
	const currentContributors = contributors.slice(
		startIndex,
		startIndex + contributorsPerPage,
	);

	// Sort contributors by contributions
	const sortedContributors = [...contributors].sort(
		(a, b) => b.contributions - a.contributions,
	);

	// Helper function to render a contributor card
	const renderContributorCard = (contributor: Contributor, index: number) => {
		const isSelected = index === selectedCardIndex;
		const cardColor = isSelected
			? 'blue'
			: contributor.contributions > 100
				? 'green'
				: contributor.contributions > 50
					? 'yellow'
					: 'gray';

		return (
			<Box
				key={contributor.id}
				flexDirection="column"
				width={30}
				height={18}
				borderStyle="round"
				borderColor={cardColor}
				padding={1}
				marginRight={2}
			>
				<Box>
					<Text bold color={isSelected ? 'blue' : 'green'}>
						{isSelected ? figures.pointer + ' ' : ''}
						{contributor.name || contributor.login}
					</Text>
				</Box>
				<Text dimColor>@{contributor.login}</Text>

				{contributor.bio && (
					<Text italic dimColor>
						{contributor.bio?.substring(0, 50)}
						{contributor.bio && contributor.bio.length > 50 ? '...' : ''}
					</Text>
				)}

				<Box
					marginTop={1}
					borderStyle="single"
					borderColor={isSelected ? cardColor : 'gray'}
					padding={1}
				>
					<Text bold>Contributions: </Text>
					<Text color="cyan">{contributor.contributions}</Text>
				</Box>

				<Box>
					<Text bold>Commits: </Text>
					<Text color="yellow">{contributor.commits || 0}</Text>
				</Box>

				<Box>
					<Text bold>Lines: </Text>
					<Text color="green">+{contributor.additions || 0}</Text>
					<Text color="red"> -{contributor.deletions || 0}</Text>
				</Box>
			</Box>
		);
	};

	return (
		<Box flexDirection="column" padding={1}>
			<Box marginBottom={1}>
				<Text bold>
					Steel Browser Contributors (Page {currentPage + 1}/{maxPages})
				</Text>
				<Text> (Use ← → arrows to navigate, q to quit)</Text>
			</Box>

			<Box flexDirection="row" marginBottom={1}>
				{currentContributors.map((contributor, index) =>
					renderContributorCard(contributor, index),
				)}
			</Box>

			<Box marginTop={1} justifyContent="center">
				<Text color="gray">{figures.arrowLeft} Previous</Text>
				<Text> | </Text>
				{Array.from({length: maxPages}).map((_, index) => (
					<Text
						key={index}
						color={index === currentPage ? 'blue' : 'gray'}
						bold={index === currentPage}
					>
						{index + 1}{' '}
					</Text>
				))}
				<Text> | </Text>
				<Text color="gray">Next {figures.arrowRight}</Text>
			</Box>

			<Box marginTop={1}>
				<Text dimColor italic>
					{figures.star} Top contributor:{' '}
					{sortedContributors[0]?.name || sortedContributors[0]?.login} with{' '}
					{sortedContributors[0]?.contributions} contributions
				</Text>
			</Box>

			<Box marginTop={1} justifyContent="center">
				<Text>
					Use{' '}
					<Text color="blue" bold>
						←/→
					</Text>{' '}
					to change pages,{' '}
					<Text color="blue" bold>
						Shift+←/→
					</Text>{' '}
					to select cards
				</Text>
			</Box>
		</Box>
	);
}
