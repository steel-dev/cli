import type { BrowserManager, ScreencastFrame } from './browser.js';
import type { Command, Response } from './types.js';
/**
 * Set the callback for screencast frames
 * This is called by the daemon to set up frame streaming
 */
export declare function setScreencastFrameCallback(callback: ((frame: ScreencastFrame) => void) | null): void;
/**
 * Convert Playwright errors to AI-friendly messages
 * @internal Exported for testing
 */
export declare function toAIFriendlyError(error: unknown, selector: string): Error;
/**
 * Execute a command and return a response
 */
export declare function executeCommand(command: Command, browser: BrowserManager): Promise<Response>;
//# sourceMappingURL=actions.d.ts.map