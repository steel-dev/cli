#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {deleteSteelProfile} from '../../utils/browser/lifecycle/profile-store.js';

export const description =
	'Delete a saved Steel browser profile (local file only)';

export const options = zod.object({
	name: zod.string().describe(
		option({
			description: 'Name of the profile to delete',
		}),
	),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function Delete({options}: Props) {
	useEffect(() => {
		async function run() {
			const deleted = await deleteSteelProfile(options.name, process.env);

			if (!deleted) {
				console.error(`Profile "${options.name}" not found.`);
				process.exit(1);
				return;
			}

			console.log(
				`Deleted profile "${options.name}". Note: Browser state on Steel servers is not affected.`,
			);
			process.exit(0);
		}

		run();
	}, [options.name]);
}
