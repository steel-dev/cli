import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';

type SteelProfileData = {
	profileId: string;
	chromeProfile?: string;
};

function getProfilesDirectory(environment: NodeJS.ProcessEnv): string {
	const configDir =
		environment.STEEL_CONFIG_DIR?.trim() ||
		path.join(os.homedir(), '.config', 'steel');
	return path.join(configDir, 'profiles');
}

function getProfilePath(name: string, environment: NodeJS.ProcessEnv): string {
	return path.join(getProfilesDirectory(environment), `${name}.json`);
}

export function validateProfileName(name: string): string | null {
	if (!name.trim()) {
		return 'Profile name cannot be empty.';
	}

	if (name.includes('/') || name.includes('\\')) {
		return `Invalid profile name "${name}". Use a name like "myapp", not a path.`;
	}

	return null;
}

export async function readSteelProfile(
	name: string,
	environment: NodeJS.ProcessEnv,
): Promise<{profileId: string; chromeProfile?: string} | null> {
	const filePath = getProfilePath(name, environment);

	try {
		const contents = await fs.readFile(filePath, 'utf-8');
		const parsed = JSON.parse(contents) as unknown;

		if (
			parsed &&
			typeof parsed === 'object' &&
			typeof (parsed as Record<string, unknown>)['profileId'] === 'string'
		) {
			const data = parsed as SteelProfileData;
			return {
				profileId: data.profileId,
				chromeProfile: data.chromeProfile,
			};
		}

		return null;
	} catch {
		return null;
	}
}

export async function writeSteelProfile(
	name: string,
	profileId: string,
	environment: NodeJS.ProcessEnv,
	chromeProfile?: string,
): Promise<void> {
	const profilesDir = getProfilesDirectory(environment);
	await fs.mkdir(profilesDir, {recursive: true});

	const filePath = getProfilePath(name, environment);
	const data: SteelProfileData = {profileId, chromeProfile};

	await fs.writeFile(filePath, JSON.stringify(data, null, 2) + '\n', 'utf-8');
}

export async function listSteelProfiles(
	environment: NodeJS.ProcessEnv,
): Promise<Array<{name: string; profileId: string}>> {
	const profilesDir = getProfilesDirectory(environment);

	let entries: string[];
	try {
		entries = await fs.readdir(profilesDir);
	} catch {
		return [];
	}

	const profiles: Array<{name: string; profileId: string}> = [];

	for (const entry of entries) {
		if (!entry.endsWith('.json')) continue;

		const name = entry.slice(0, -5);
		const filePath = path.join(profilesDir, entry);

		try {
			const contents = await fs.readFile(filePath, 'utf-8');
			const parsed = JSON.parse(contents) as unknown;

			if (
				parsed &&
				typeof parsed === 'object' &&
				typeof (parsed as Record<string, unknown>)['profileId'] === 'string'
			) {
				profiles.push({
					name,
					profileId: (parsed as SteelProfileData).profileId,
				});
			}
		} catch {
			// Skip corrupt files
		}
	}

	return profiles;
}

export async function deleteSteelProfile(
	name: string,
	environment: NodeJS.ProcessEnv,
): Promise<boolean> {
	const filePath = getProfilePath(name, environment);

	try {
		await fs.unlink(filePath);
		return true;
	} catch {
		return false;
	}
}
