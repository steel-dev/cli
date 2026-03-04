import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';

type SteelProfileData = {
	profileId: string;
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
): Promise<{profileId: string} | null> {
	const filePath = getProfilePath(name, environment);

	try {
		const contents = await fs.readFile(filePath, 'utf-8');
		const parsed = JSON.parse(contents) as unknown;

		if (
			parsed &&
			typeof parsed === 'object' &&
			typeof (parsed as Record<string, unknown>)['profileId'] === 'string'
		) {
			return {profileId: (parsed as SteelProfileData).profileId};
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
): Promise<void> {
	const profilesDir = getProfilesDirectory(environment);
	await fs.mkdir(profilesDir, {recursive: true});

	const filePath = getProfilePath(name, environment);
	const data: SteelProfileData = {profileId};

	await fs.writeFile(filePath, JSON.stringify(data, null, 2) + '\n', 'utf-8');
}
