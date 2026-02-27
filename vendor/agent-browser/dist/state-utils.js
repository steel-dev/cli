/**
 * Shared utilities for session state management.
 */
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { getEncryptionKey, encryptData, decryptData, isEncryptedPayload, ENCRYPTION_KEY_ENV, } from './encryption.js';
/**
 * Get the session persistence directory.
 * Located at ~/.agent-browser/sessions/
 */
export function getSessionsDir() {
    return path.join(os.homedir(), '.agent-browser', 'sessions');
}
/**
 * Ensure the sessions directory exists with proper permissions.
 * Creates directory with mode 0o700 (owner only).
 */
export function ensureSessionsDir() {
    const sessionsDir = getSessionsDir();
    if (!fs.existsSync(sessionsDir)) {
        fs.mkdirSync(sessionsDir, { recursive: true, mode: 0o700 });
    }
    return sessionsDir;
}
/**
 * Validate a session ID to prevent path traversal attacks.
 * Only allows alphanumeric characters, hyphens, and underscores.
 */
function isValidSessionId(id) {
    return /^[a-zA-Z0-9_-]+$/.test(id);
}
/**
 * Validate a session name for safety (no path traversal).
 * Only allows alphanumeric characters, dashes, and underscores.
 * This validation is critical for security - the daemon reads session names
 * from environment variables which can be set by attackers bypassing CLI validation.
 */
export function isValidSessionName(name) {
    return /^[a-zA-Z0-9_-]+$/.test(name);
}
/**
 * Get the auto-save state file path for a session.
 * Pattern: {SESSION_NAME}-{SESSION_ID}.json
 *
 * @param sessionName - The session name (e.g., "twitter")
 * @param sessionId - The session ID (e.g., "default" or "agent1")
 * @returns Full path to the state file, or null if sessionName is empty
 * @throws Error if sessionName or sessionId contains invalid characters (path traversal prevention)
 */
export function getAutoStateFilePath(sessionName, sessionId) {
    if (!sessionName)
        return null;
    // SECURITY: Validate sessionName to prevent path traversal attacks.
    // The daemon reads AGENT_BROWSER_SESSION_NAME from environment which
    // can be set directly by attackers, bypassing CLI validation.
    if (!isValidSessionName(sessionName)) {
        throw new Error(`Invalid session name '${sessionName}'. Only alphanumeric characters, hyphens, and underscores are allowed.`);
    }
    if (!isValidSessionId(sessionId)) {
        throw new Error(`Invalid session ID '${sessionId}'. Only alphanumeric characters, hyphens, and underscores are allowed.`);
    }
    const sessionsDir = ensureSessionsDir();
    return path.join(sessionsDir, `${sessionName}-${sessionId}.json`);
}
/**
 * Check if an auto-state file exists for a session.
 */
export function autoStateFileExists(sessionName, sessionId) {
    const filePath = getAutoStateFilePath(sessionName, sessionId);
    return filePath ? fs.existsSync(filePath) : false;
}
/**
 * Write state data to file, encrypting if encryption key is available.
 *
 * @param filepath - Path to write the state file
 * @param data - State data object to write
 * @returns Object indicating whether the file was encrypted
 */
export function writeStateFile(filepath, data) {
    const key = getEncryptionKey();
    const jsonData = JSON.stringify(data, null, 2);
    if (key) {
        const encrypted = encryptData(jsonData, key);
        fs.writeFileSync(filepath, JSON.stringify(encrypted, null, 2));
        return { encrypted: true };
    }
    fs.writeFileSync(filepath, jsonData);
    return { encrypted: false };
}
/**
 * Read state data from file, decrypting if necessary.
 *
 * @param filepath - Path to the state file
 * @returns Object containing the data and whether it was encrypted
 * @throws Error if file is encrypted but no key is available
 */
export function readStateFile(filepath) {
    const content = fs.readFileSync(filepath, 'utf-8');
    const parsed = JSON.parse(content);
    if (isEncryptedPayload(parsed)) {
        const key = getEncryptionKey();
        if (!key) {
            throw new Error(`State file is encrypted but ${ENCRYPTION_KEY_ENV} is not set. ` +
                `Set the environment variable to decrypt.`);
        }
        const decrypted = decryptData(parsed, key);
        return { data: JSON.parse(decrypted), wasEncrypted: true };
    }
    return { data: parsed, wasEncrypted: false };
}
/**
 * List all state files in the sessions directory.
 * @returns Array of filenames ending in .json
 */
export function listStateFiles() {
    const sessionsDir = getSessionsDir();
    if (!fs.existsSync(sessionsDir)) {
        return [];
    }
    return fs.readdirSync(sessionsDir).filter((f) => f.endsWith('.json'));
}
/**
 * Clean up state files older than specified days.
 * @param days - Maximum age in days (files older than this are deleted)
 * @returns Array of deleted filenames
 */
export function cleanupExpiredStates(days) {
    if (days <= 0)
        return [];
    const sessionsDir = getSessionsDir();
    if (!fs.existsSync(sessionsDir)) {
        return [];
    }
    const now = Date.now();
    const maxAge = days * 24 * 60 * 60 * 1000;
    const deleted = [];
    const files = listStateFiles();
    for (const file of files) {
        const filepath = path.join(sessionsDir, file);
        try {
            const stats = fs.statSync(filepath);
            if (now - stats.mtime.getTime() > maxAge) {
                fs.unlinkSync(filepath);
                deleted.push(file);
            }
        }
        catch {
            // Ignore individual file errors
        }
    }
    return deleted;
}
const DANGEROUS_KEYS = ['__proto__', 'constructor', 'prototype'];
/**
 * Safely merge headers without prototype pollution risk.
 * Filters out dangerous keys like __proto__, constructor, prototype.
 * @param base - Base headers object
 * @param override - Headers to merge (takes precedence)
 * @returns Merged headers object (null-prototype)
 */
export function safeHeaderMerge(base, override) {
    const result = Object.create(null);
    for (const [key, value] of Object.entries(base)) {
        if (!DANGEROUS_KEYS.includes(key)) {
            result[key] = value;
        }
    }
    for (const [key, value] of Object.entries(override)) {
        if (!DANGEROUS_KEYS.includes(key)) {
            result[key] = value;
        }
    }
    return result;
}
// Re-export encryption utilities
export { getEncryptionKey, encryptData, decryptData, isEncryptedPayload, ENCRYPTION_KEY_ENV, };
//# sourceMappingURL=state-utils.js.map