/**
 * Encryption utilities for state file protection using AES-256-GCM.
 */
export declare const ENCRYPTION_ALGORITHM = "aes-256-gcm";
export declare const ENCRYPTION_KEY_ENV = "AGENT_BROWSER_ENCRYPTION_KEY";
export declare const IV_LENGTH = 12;
/**
 * Encrypted payload structure.
 */
export interface EncryptedPayload {
    version: 1;
    encrypted: true;
    iv: string;
    authTag: string;
    data: string;
}
export declare function getKeyFilePath(): string;
/**
 * Restrict file permissions to the current user only.
 * On Unix, the caller should use `mode: 0o600` when writing. This function
 * handles Windows where Node's mode parameter is ignored.
 */
export declare function restrictFilePermissions(filePath: string): void;
/**
 * Restrict directory permissions to the current user only.
 * On Unix, the caller should use `mode: 0o700` when creating. This function
 * handles Windows where Node's mode parameter is ignored.
 */
export declare function restrictDirPermissions(dirPath: string): void;
/**
 * Get encryption key from environment variable or key file.
 * The key should be a 32-byte (256-bit) hex-encoded string (64 characters).
 * Generate with: openssl rand -hex 32
 *
 * Checks (in order):
 * 1. AGENT_BROWSER_ENCRYPTION_KEY env var
 * 2. ~/.agent-browser/.encryption-key file
 *
 * @returns Buffer containing the key, or null if not available
 */
export declare function getEncryptionKey(): Buffer | null;
/**
 * Ensure an encryption key is available, auto-generating one if needed.
 * On first call without an existing key, generates a random 256-bit key
 * and writes it to ~/.agent-browser/.encryption-key (mode 0600).
 */
export declare function ensureEncryptionKey(): Buffer;
/**
 * Encrypt data using AES-256-GCM.
 * Returns a JSON-serializable payload with IV, auth tag, and encrypted data.
 *
 * @param plaintext - The string to encrypt
 * @param key - The 256-bit encryption key
 * @returns Encrypted payload object
 */
export declare function encryptData(plaintext: string, key: Buffer): EncryptedPayload;
/**
 * Decrypt data using AES-256-GCM.
 *
 * @param payload - The encrypted payload object
 * @param key - The 256-bit encryption key
 * @returns Decrypted plaintext string
 * @throws Error if decryption fails (wrong key, tampered data, etc.)
 */
export declare function decryptData(payload: EncryptedPayload, key: Buffer): string;
/**
 * Check if a parsed JSON object is an encrypted payload.
 *
 * @param data - The object to check
 * @returns True if the object is a valid encrypted payload
 */
export declare function isEncryptedPayload(data: unknown): data is EncryptedPayload;
//# sourceMappingURL=encryption.d.ts.map