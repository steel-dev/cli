import type { BrowserContext } from 'playwright-core';
/**
 * Checks whether a hostname matches one of the allowed domain patterns.
 * Patterns support exact match ("example.com") and wildcard prefix ("*.example.com").
 */
export declare function isDomainAllowed(hostname: string, allowedDomains: string[]): boolean;
export declare function parseDomainList(raw: string): string[];
/**
 * Build the init script source that monkey-patches WebSocket, EventSource,
 * and navigator.sendBeacon to block connections to non-allowed domains.
 * Exported for testing.
 */
export declare function buildWebSocketFilterScript(allowedDomains: string[]): string;
/**
 * Installs a context-level route that enforces the domain allowlist.
 * Both document navigations and sub-resource requests (scripts, images, fetch, etc.)
 * to non-allowed domains are blocked, preventing data exfiltration.
 * Non-http(s) schemes (data:, blob:, etc.) are allowed for sub-resources
 * but blocked for document navigations.
 *
 * Also installs an init script that patches WebSocket, EventSource, and
 * navigator.sendBeacon to block connections to non-allowed domains. This is
 * a best-effort defense: if eval is permitted by action policy, page scripts
 * could theoretically restore the originals. Denying the eval action
 * category closes that loophole.
 */
export declare function installDomainFilter(context: BrowserContext, allowedDomains: string[]): Promise<void>;
//# sourceMappingURL=domain-filter.d.ts.map