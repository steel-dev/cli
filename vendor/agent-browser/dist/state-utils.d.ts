/**
 * Shared utilities for session state management.
 */
import { getEncryptionKey, encryptData, decryptData, isEncryptedPayload, type EncryptedPayload, ENCRYPTION_KEY_ENV } from './encryption.js';
/**
 * Get the session persistence directory.
 * Located at ~/.agent-browser/sessions/
 */
export declare function getSessionsDir(): string;
/**
 * Ensure the sessions directory exists with proper permissions.
 * Creates directory with mode 0o700 (owner only).
 */
export declare function ensureSessionsDir(): string;
/**
 * Validate a session name for safety (no path traversal).
 * Only allows alphanumeric characters, dashes, and underscores.
 * This validation is critical for security - the daemon reads session names
 * from environment variables which can be set by attackers bypassing CLI validation.
 */
export declare function isValidSessionName(name: string): boolean;
/**
 * Get the auto-save state file path for a session.
 * Pattern: {SESSION_NAME}-{SESSION_ID}.json
 *
 * @param sessionName - The session name (e.g., "twitter")
 * @param sessionId - The session ID (e.g., "default" or "agent1")
 * @returns Full path to the state file, or null if sessionName is empty
 * @throws Error if sessionName or sessionId contains invalid characters (path traversal prevention)
 */
export declare function getAutoStateFilePath(sessionName: string, sessionId: string): string | null;
/**
 * Check if an auto-state file exists for a session.
 */
export declare function autoStateFileExists(sessionName: string, sessionId: string): boolean;
/**
 * Write state data to file, encrypting if encryption key is available.
 *
 * @param filepath - Path to write the state file
 * @param data - State data object to write
 * @returns Object indicating whether the file was encrypted
 */
export declare function writeStateFile(filepath: string, data: object): {
    encrypted: boolean;
};
/**
 * Read state data from file, decrypting if necessary.
 *
 * @param filepath - Path to the state file
 * @returns Object containing the data and whether it was encrypted
 * @throws Error if file is encrypted but no key is available
 */
export declare function readStateFile(filepath: string): {
    data: object;
    wasEncrypted: boolean;
};
/**
 * List all state files in the sessions directory.
 * @returns Array of filenames ending in .json
 */
export declare function listStateFiles(): string[];
/**
 * Clean up state files older than specified days.
 * @param days - Maximum age in days (files older than this are deleted)
 * @returns Array of deleted filenames
 */
export declare function cleanupExpiredStates(days: number): string[];
/**
 * Safely merge headers without prototype pollution risk.
 * Filters out dangerous keys like __proto__, constructor, prototype.
 * @param base - Base headers object
 * @param override - Headers to merge (takes precedence)
 * @returns Merged headers object (null-prototype)
 */
export declare function safeHeaderMerge(base: Record<string, string>, override: Record<string, string>): Record<string, string>;
export { getEncryptionKey, encryptData, decryptData, isEncryptedPayload, type EncryptedPayload, ENCRYPTION_KEY_ENV, };
//# sourceMappingURL=state-utils.d.ts.map