#!/usr/bin/env node

import {useEffect} from 'react';
import zod from 'zod';
import {option} from 'pastel';
import {listSteelProfiles} from '../../utils/browser/lifecycle/profile-store.js';

export const description = 'List all saved Steel browser profiles';

export const options = zod.object({
	json: zod
		.boolean()
		.describe(
			option({
				description: 'Output profiles as JSON',
			}),
		)
		.optional(),
});

type Props = {
	options: zod.infer<typeof options>;
};

export default function List({options}: Props) {
	useEffect(() => {
		async function run() {
			const profiles = await listSteelProfiles(process.env);

			if (options.json) {
				console.log(JSON.stringify(profiles, null, 2));
				process.exit(0);
				return;
			}

			if (profiles.length === 0) {
				console.log(
					'No profiles found. Use --profile <name> with steel browser start to create one.',
				);
				process.exit(0);
				return;
			}

			const nameWidth = Math.max(4, ...profiles.map(p => p.name.length));
			console.log(`${'NAME'.padEnd(nameWidth)}  PROFILE_ID`);
			for (const profile of profiles) {
				console.log(`${profile.name.padEnd(nameWidth)}  ${profile.profileId}`);
			}

			process.exit(0);
		}

		run();
	}, [options.json]);
}
