#!/usr/bin/env node

import React, {useEffect, useState} from 'react';
import fs from 'fs';
import path from 'path';
import {Box, Text} from 'ink';
import Spinner from 'ink-spinner';
import zod from 'zod';
import {option} from 'pastel';
import {CACHE_DIR} from '../utils/constants.js';

export const description = 'Manage Steel CLI cache';

export const options = zod.object({
	clean: zod
		.boolean()
		.describe(
			option({
				description: 'Remove all cached files and directories',
				alias: 'c',
			}),
		)
		.optional(),
});

export type Options = zod.infer<typeof options>;

type Props = {
	options: Options;
};

export default function Cache({options}: Props) {
	const [status, setStatus] = useState<'idle' | 'cleaning' | 'done' | 'error'>(
		'idle',
	);
	const [message, setMessage] = useState('');
	const [deletedItems, setDeletedItems] = useState<string[]>([]);

	useEffect(() => {
		const cleanCache = async () => {
			try {
				if (!options.clean) {
					setStatus('idle');
					setMessage('Use --clean to remove all cached files and directories');
					return;
				}

				setStatus('cleaning');
				setMessage('Cleaning cache directory...');

				// Check if cache directory exists
				if (!fs.existsSync(CACHE_DIR)) {
					fs.mkdirSync(CACHE_DIR, {recursive: true});
					setStatus('done');
					setMessage('Cache directory is already empty');
					return;
				}

				// Read all items in the cache directory
				const items = fs.readdirSync(CACHE_DIR);
				const deleted: string[] = [];

				// Delete each item in the cache directory
				for (const item of items) {
					const itemPath = path.join(CACHE_DIR, item);
					const isDirectory = fs.statSync(itemPath).isDirectory();

					try {
						if (isDirectory) {
							fs.rmSync(itemPath, {recursive: true, force: true});
						} else {
							fs.unlinkSync(itemPath);
						}
						deleted.push(item);
					} catch (err) {
						console.error(`Failed to delete ${itemPath}:`, err);
					}
				}

				setDeletedItems(deleted);
				setStatus('done');
				setMessage(
					`Cache cleaned successfully: removed ${deleted.length} items`,
				);
			} catch (error) {
				setStatus('error');
				setMessage(`Error cleaning cache: ${error.message}`);
			}
		};

		cleanCache();
	}, [options.clean]);

	return (
		<Box flexDirection="column">
			<Box marginBottom={1}>
				<Text bold>Steel CLI Cache Manager</Text>
			</Box>

			<Box>
				{status === 'cleaning' ? (
					<>
						<Text color="yellow">
							<Spinner type="dots" />
						</Text>
						<Text> {message}</Text>
					</>
				) : status === 'done' ? (
					<Text color="green">{message}</Text>
				) : status === 'error' ? (
					<Text color="red">{message}</Text>
				) : (
					<Text>{message}</Text>
				)}
			</Box>

			{status === 'done' && deletedItems.length > 0 && (
				<Box flexDirection="column" marginTop={1}>
					<Text bold>Removed items:</Text>
					{deletedItems.map((item, index) => (
						<Text key={index} color="gray">
							- {item}
						</Text>
					))}
					<Box marginTop={1}>
						<Text>
							Cache directory: <Text color="cyan">{CACHE_DIR}</Text>
						</Text>
					</Box>
				</Box>
			)}
		</Box>
	);
}
