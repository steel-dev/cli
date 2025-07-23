import crypto from 'crypto';
import fs from 'fs';

export function hashDeps(depsFile: string): string {
	return crypto
		.createHash('sha256')
		.update(fs.readFileSync(depsFile))
		.digest('hex')
		.slice(0, 12);
}

export function hashString(input: string): string {
	return crypto.createHash('sha256').update(input).digest('hex').slice(0, 12);
}
