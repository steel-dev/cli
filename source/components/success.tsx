import React from 'react';
import {Box, Text} from 'ink';

export default function Success({
	title,
	message,
	details = [],
	condition = false,
}: {
	title?: string;
	message?: string;
	details?: string[];
	condition?: boolean;
}) {
	return condition ? (
		<Box flexDirection="column" marginTop={1} marginBottom={1}>
			{/* Success Header */}
			<Box marginBottom={1}>
				<Text bold color="green">
					✅ {title || 'Success'}
				</Text>
			</Box>

			{/* Main Message */}
			{message && (
				<Box paddingLeft={2} marginBottom={details.length > 0 ? 1 : 0}>
					<Text color="green">{message}</Text>
				</Box>
			)}

			{/* Details List */}
			{details.length > 0 && (
				<Box flexDirection="column">
					{details.map((detail, index) => (
						<Box key={index} paddingLeft={4} marginBottom={0}>
							<Box width={2}>
								<Text color="green">└─</Text>
							</Box>
							<Text>{detail}</Text>
						</Box>
					))}
				</Box>
			)}
		</Box>
	) : null;
}
