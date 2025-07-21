import React from 'react';
import {Box, Text} from 'ink';
import {getGlobalUpdateInfo} from '../utils/update.js';

export default function UpdateNotification() {
	const updateInfo = getGlobalUpdateInfo();

	if (!updateInfo || !updateInfo.hasUpdate) {
		return null;
	}

	return (
		<Box borderStyle="round" borderColor="yellow" paddingX={2} paddingY={1}>
			<Box flexDirection="column">
				<Box flexDirection="row" gap={1}>
					<Text color="yellow" bold>
						💡 CLI Update Available
					</Text>
					<Text>
						(<Text color="gray">v{updateInfo.current}</Text> →{' '}
						<Text color="green">v{updateInfo.latest}</Text>)
					</Text>
				</Box>
				<Box flexDirection="row" marginLeft={3}>
					<Text color="gray">
						Run{' '}
						<Text color="yellow" bold>
							steel update
						</Text>{' '}
						to update
					</Text>
				</Box>
			</Box>
		</Box>
	);
}
