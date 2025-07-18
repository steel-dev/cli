import React, {ReactNode} from 'react';
import {Box, Text} from 'ink';
import figures from 'figures';

export type Variant = 'info' | 'success' | 'warning' | 'failed';

export type CalloutProps = {
	variant: Variant;
	title: string;
	children?: ReactNode;
};

const COLORS: Record<Variant, {border: string; label: string}> = {
	info: {border: 'blue', label: 'INFO'},
	success: {border: 'green', label: 'SUCCEEDED'},
	warning: {border: 'yellow', label: 'WARNING'},
	failed: {border: 'red', label: 'FAILED'},
};

export default function Callout({variant, title, children}: CalloutProps) {
	const {border, label} = COLORS[variant];
	const icon =
		variant === 'success'
			? figures.tick
			: variant === 'failed'
				? figures.cross
				: variant === 'warning'
					? figures.warning
					: figures.info;

	// The label area: icon + label, with padding on the right, all on one background
	return (
		<Box
			flexDirection="column"
			borderStyle="round"
			borderColor={border}
			paddingX={2}
			paddingTop={1}
			paddingBottom={1}
		>
			<Box>
				{/* Icon + label + right padding, all on one background */}
				<Text backgroundColor={border} color="black" bold>
					{' '}
					{icon} {label}{' '}
				</Text>
				{/* Title, separated by a space, not on colored background */}
				{title && <Text bold> {title}</Text>}
			</Box>
			{/* Add a blank line between the header and the message */}
			{children && (
				<>
					<Text> </Text>
					<Text color="gray">
						{typeof children === 'string'
							? children
									// First, split out URLs as before
									.split(/(http:\/\/[^\s]+|https:\/\/[^\s]+)/g)
									.map((part, i) => {
										// If it's a URL, style as link
										if (/^(http:\/\/[^\s]+|https:\/\/[^\s]+)$/.test(part)) {
											return (
												<Text key={`url-${i}`} color="blue" underline>
													{part}
												</Text>
											);
										}
										// Now, for each non-URL part, split out code snippets
										const codeSplit = part.split(/(`[^`]+`)/g);
										return codeSplit.map((subpart, j) => {
											const codeMatch = /^`([^`]+)`$/.exec(subpart);
											if (codeMatch) {
												return (
													<Text
														key={`code-${i}-${j}`}
														backgroundColor="gray"
														color="white"
													>
														{' '}
														{codeMatch[1]}{' '}
													</Text>
												);
											}
											return <Text key={`text-${i}-${j}`}>{subpart}</Text>;
										});
									})
							: children}
					</Text>
				</>
			)}
		</Box>
	);
}
