import React from 'react';
import {Box, Text} from 'ink';

export function Error({
	title,
	message,
	details = [],
}: {
	title?: string;
	message?: string;
	details?: string[];
}) {
	return (
		<Box flexDirection="column" marginTop={1} marginBottom={1}>
			{/* Error Header */}
			<Box marginBottom={1}>
				<Text bold color="red">
					❌ {title || 'Error'}
				</Text>
			</Box>

			{/* Main Message */}
			{message && (
				<Box paddingLeft={2} marginBottom={details.length > 0 ? 1 : 0}>
					<Text color="red">{message}</Text>
				</Box>
			)}

			{/* Details List */}
			{details.length > 0 && (
				<Box flexDirection="column">
					{details.map((detail, index) => (
						<Box key={index} paddingLeft={4} marginBottom={0}>
							<Box width={2}>
								<Text color="red">└─</Text>
							</Box>
							<Text>{detail}</Text>
						</Box>
					))}
				</Box>
			)}
		</Box>
	);
}
