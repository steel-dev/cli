/**
 * Enhanced snapshot with element refs for deterministic element selection.
 *
 * This module generates accessibility snapshots with embedded refs that can be
 * used to click/fill/interact with elements without re-querying the DOM.
 *
 * Example output:
 *   - heading "Example Domain" [ref=e1] [level=1]
 *   - paragraph: Some text content
 *   - button "Submit" [ref=e2]
 *   - textbox "Email" [ref=e3]
 *
 * Usage:
 *   agent-browser snapshot              # Full snapshot
 *   agent-browser snapshot -i           # Interactive elements only
 *   agent-browser snapshot --depth 3    # Limit depth
 *   agent-browser click @e2             # Click element by ref
 */
import type { Page } from 'playwright-core';
export interface RefMap {
    [ref: string]: {
        selector: string;
        role: string;
        name?: string;
        /** Index for disambiguation when multiple elements have same role+name */
        nth?: number;
    };
}
export interface EnhancedSnapshot {
    tree: string;
    refs: RefMap;
}
export interface SnapshotOptions {
    /** Only include interactive elements (buttons, links, inputs, etc.) */
    interactive?: boolean;
    /** Include cursor-interactive elements (cursor:pointer, onclick, tabindex) */
    cursor?: boolean;
    /** Maximum depth of tree to include (0 = root only) */
    maxDepth?: number;
    /** Remove structural elements without meaningful content */
    compact?: boolean;
    /** CSS selector to scope the snapshot */
    selector?: string;
}
/**
 * Reset ref counter (call at start of each snapshot)
 */
export declare function resetRefs(): void;
/**
 * Get enhanced snapshot with refs and optional filtering
 */
export declare function getEnhancedSnapshot(page: Page, options?: SnapshotOptions): Promise<EnhancedSnapshot>;
/**
 * Parse a ref from command argument (e.g., "@e1" -> "e1")
 */
export declare function parseRef(arg: string): string | null;
/**
 * Get snapshot statistics
 */
export declare function getSnapshotStats(tree: string, refs: RefMap): {
    lines: number;
    chars: number;
    tokens: number;
    refs: number;
    interactive: number;
};
//# sourceMappingURL=snapshot.d.ts.map