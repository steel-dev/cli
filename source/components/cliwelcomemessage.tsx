import React from 'react';
import {Box, Text, useStdout} from 'ink';
import chalk from 'chalk';

// ASCII art lines for the logo
const logoLines = [
	chalk.yellowBright(' @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ '),
	chalk.yellowBright('@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@'),
	chalk.yellowBright('@@@@@@@@@9999999999999@@@@@@@@@'),
	chalk.yellowBright('@@@@@[                   ]@@@@@'),
	chalk.yellowBright('@@@@[                      @@@@'),
	chalk.yellowBright('@@@@     @@@@@@@@@@@@@     @@@@'),
	chalk.yellowBright('@@@@B              @@@     @@@@'),
	chalk.yellowBright('@@@@@@               @     @@@@'),
	chalk.yellowBright('@@@@@@@@@@@@@@@@     @     @@@@'),
	chalk.yellowBright('@@@@                 @     @@@@'),
	chalk.yellowBright('@@@@                @@     @@@@'),
	chalk.yellowBright('@@@@@@@@@@@@@@@@g@@@@@@@@@@@@@@'),
	chalk.yellowBright('@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@'),
	chalk.yellowBright(' @@@@@@@@@@@@@@@@@@@@@@@@@@@@@ '),
];

// Text content that will be on the right or below
const textContent = [
	{
		text: 'Humans use Chrome, Agents use Steel.',
		color: 'cyan' as const,
		lineIndex: 2, // Position next to 3rd logo line when side-by-side
	},
	{
		text: 'Steel is an open-source browser API purpose-built for AI agents.',
		color: undefined,
		lineIndex: 6, // Position next to 7th logo line when side-by-side
	},
	{
		text: 'Give one or 1,000 agents the ability to interact with any website.',
		color: undefined,
		lineIndex: 7, // Position next to 8th logo line when side-by-side
	},
	{
		text: `${chalk.green('Documentation:')} ${chalk.blueBright('https://docs.steel.dev/')}`,
		color: undefined,
		lineIndex: 11, // Position next to 12th logo line when side-by-side
	},
	{
		text: `${chalk.green('GitHub:')} ${chalk.blueBright('https://github.com/steel-dev/steel-browser')}`,
		color: undefined,
		lineIndex: 12, // Position next to 13th logo line when side-by-side
	},
];

export default function CLIWelcomeMessage() {
	const {stdout} = useStdout();

	// Default to 80 columns if stdout is not available
	const terminalWidth = stdout?.columns || 80;

	// Logo width is approximately 31 characters + some padding
	const logoWidth = 35;
	// Minimum width needed for side-by-side layout
	const minSideBySideWidth = 90;

	const useSideBySideLayout = terminalWidth >= minSideBySideWidth;

	if (useSideBySideLayout) {
		// Side-by-side layout for wider terminals
		const availableTextWidth = terminalWidth - logoWidth - 4; // 4 for padding

		return (
			<Box flexDirection="column" marginBottom={2}>
				<Box flexDirection="column">
					{logoLines.map((logoLine, index) => {
						const textForThisLine = textContent.find(
							item => item.lineIndex === index,
						);

						return (
							<Box key={index} flexDirection="row">
								<Box width={logoWidth}>
									<Text>{logoLine}</Text>
								</Box>
								{textForThisLine && (
									<Box width={availableTextWidth} flexWrap="wrap">
										<Text color={textForThisLine.color}>
											{textForThisLine.text}
										</Text>
									</Box>
								)}
							</Box>
						);
					})}
				</Box>
				<Box marginTop={1}>
					<Text color="gray">
						{'─'.repeat(Math.min(50, terminalWidth - 4))}
					</Text>
				</Box>
			</Box>
		);
	} else {
		// Stacked layout for narrower terminals
		return (
			<Box flexDirection="column" marginBottom={2}>
				{/* Logo section */}
				<Box flexDirection="column" marginBottom={1}>
					{logoLines.map((logoLine, index) => (
						<Text key={index}>{logoLine}</Text>
					))}
				</Box>

				{/* Text content section with wrapping */}
				<Box flexDirection="column" marginBottom={1}>
					{textContent.map((item, index) => (
						<Box
							key={index}
							marginBottom={index === textContent.length - 1 ? 0 : 1}
						>
							<Box width={terminalWidth - 4} flexWrap="wrap">
								<Text color={item.color}>{item.text}</Text>
							</Box>
						</Box>
					))}
				</Box>

				<Box marginTop={1}>
					<Text color="gray">
						{'─'.repeat(Math.min(50, terminalWidth - 4))}
					</Text>
				</Box>
			</Box>
		);
	}
}
