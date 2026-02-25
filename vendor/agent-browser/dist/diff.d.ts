import type { BrowserContext } from 'playwright-core';
import type { DiffSnapshotData, DiffScreenshotData } from './types.js';
/**
 * Produce a unified diff string and stats from two snapshot texts.
 */
export declare function diffSnapshots(before: string, after: string): DiffSnapshotData;
/**
 * Compare two image buffers using the browser's Canvas API for pixel comparison.
 * Uses an isolated blank page to avoid CSP interference or DOM side effects on the
 * user's page. Images are served via intercepted routes to avoid large base64 payloads
 * through page.evaluate (which can be slow or hit CDP message size limits).
 */
export declare function diffScreenshots(context: BrowserContext, baselineBuffer: Buffer, currentBuffer: Buffer, opts: {
    threshold?: number;
    outputPath?: string;
    baselineMime?: string;
}): Promise<DiffScreenshotData>;
//# sourceMappingURL=diff.d.ts.map