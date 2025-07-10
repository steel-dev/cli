import {Box, Text, useStdout} from 'ink';

export default function ProgressBar({
	percent = 0,
	left = '[',
	right = ']',
	character = '#',
}: {
	percent: number;
	left?: string;
	right?: string;
	character?: string;
}) {
	const {stdout} = useStdout();

	if (!stdout) return null;

	const totalWidth = stdout.columns - 20 || 80;
	const barWidth = totalWidth - left.length - right.length - 4; // buffer for spacing
	const filled = Math.round(barWidth * percent);
	const empty = barWidth - filled > 0 ? barWidth - filled : 0;
	return (
		<Box>
			<Text>{left}</Text>
			<Text color="green">{character.repeat(filled)}</Text>
			<Text dimColor>{character.repeat(empty)}</Text>
			<Text>{right}</Text>
		</Box>
	);
}
