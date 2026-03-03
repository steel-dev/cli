import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync, unlinkSync, } from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { getEncryptionKey, ensureEncryptionKey, encryptData, decryptData, isEncryptedPayload, getKeyFilePath, restrictFilePermissions, restrictDirPermissions, } from './encryption.js';
const AUTH_DIR = 'auth';
function getAuthDir() {
    const dir = path.join(os.homedir(), '.agent-browser', AUTH_DIR);
    if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true, mode: 0o700 });
        restrictDirPermissions(dir);
    }
    return dir;
}
const SAFE_NAME_RE = /^[a-zA-Z0-9_-]+$/;
function validateProfileName(name) {
    if (!SAFE_NAME_RE.test(name)) {
        throw new Error(`Invalid auth profile name '${name}': only alphanumeric characters, hyphens, and underscores are allowed`);
    }
}
function profilePath(name) {
    validateProfileName(name);
    return path.join(getAuthDir(), `${name}.json`);
}
function readProfile(name) {
    const p = profilePath(name);
    if (!existsSync(p))
        return null;
    const raw = readFileSync(p, 'utf-8');
    const parsed = JSON.parse(raw);
    if (isEncryptedPayload(parsed)) {
        const key = getEncryptionKey();
        if (!key) {
            throw new Error(`Encryption key required to read encrypted auth profiles. ` +
                `Set AGENT_BROWSER_ENCRYPTION_KEY or ensure ${getKeyFilePath()} exists.`);
        }
        const decrypted = decryptData(parsed, key);
        return JSON.parse(decrypted);
    }
    return parsed;
}
function writeProfile(profile) {
    const key = ensureEncryptionKey();
    const serialized = JSON.stringify(profile, null, 2);
    const encrypted = encryptData(serialized, key);
    const filePath = profilePath(profile.name);
    writeFileSync(filePath, JSON.stringify(encrypted, null, 2), {
        mode: 0o600,
    });
    restrictFilePermissions(filePath);
}
export function saveAuthProfile(opts) {
    const existing = readProfile(opts.name);
    const profile = {
        name: opts.name,
        url: opts.url,
        username: opts.username,
        password: opts.password,
        usernameSelector: opts.usernameSelector,
        passwordSelector: opts.passwordSelector,
        submitSelector: opts.submitSelector,
        createdAt: existing?.createdAt ?? new Date().toISOString(),
        lastLoginAt: existing?.lastLoginAt,
    };
    writeProfile(profile);
    return {
        name: profile.name,
        url: profile.url,
        username: profile.username,
        createdAt: profile.createdAt,
        lastLoginAt: profile.lastLoginAt,
        updated: existing !== null,
    };
}
export function getAuthProfile(name) {
    return readProfile(name);
}
export function getAuthProfileMeta(name) {
    const profile = readProfile(name);
    if (!profile)
        return null;
    return {
        name: profile.name,
        url: profile.url,
        username: profile.username,
        createdAt: profile.createdAt,
        lastLoginAt: profile.lastLoginAt,
    };
}
export function listAuthProfiles() {
    const dir = getAuthDir();
    const files = readdirSync(dir).filter((f) => f.endsWith('.json'));
    const profiles = [];
    for (const file of files) {
        const name = file.replace(/\.json$/, '');
        try {
            const meta = getAuthProfileMeta(name);
            if (meta)
                profiles.push(meta);
        }
        catch {
            profiles.push({
                name,
                url: '(encrypted)',
                username: '(encrypted)',
                createdAt: '(unknown)',
            });
        }
    }
    return profiles;
}
export function deleteAuthProfile(name) {
    const p = profilePath(name);
    if (!existsSync(p))
        return false;
    unlinkSync(p);
    return true;
}
export function updateLastLogin(name) {
    const profile = readProfile(name);
    if (profile) {
        profile.lastLoginAt = new Date().toISOString();
        writeProfile(profile);
    }
}
//# sourceMappingURL=auth-vault.js.map