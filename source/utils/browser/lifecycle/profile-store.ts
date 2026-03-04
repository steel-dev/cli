import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';

type SteelProfileData = {
	profileId: string;
	updatedAt: string;
};

function expandProfileDir(dir: string): string {
	if (dir.startsWith('~')) {
		return path.join(os.homedir(), dir.slice(1));
	}

	return dir;
}

export async function readSteelProfile(
	dir: string,
): Promise<{profileId: string} | null> {
	const expandedDir = expandProfileDir(dir);
	const filePath = path.join(expandedDir, '.steel.json');

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
	dir: string,
	profileId: string,
): Promise<void> {
	const expandedDir = expandProfileDir(dir);

	await fs.mkdir(expandedDir, {recursive: true});

	const filePath = path.join(expandedDir, '.steel.json');
	const data: SteelProfileData = {
		profileId,
		updatedAt: new Date().toISOString(),
	};

	await fs.writeFile(filePath, JSON.stringify(data, null, 2) + '\n', 'utf-8');
}
