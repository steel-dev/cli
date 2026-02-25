/**
 * iOS command execution - mirrors actions.ts but for iOS Safari via Appium.
 * Provides 1:1 command parity where possible.
 */
import type { IOSManager } from './ios-manager.js';
import type { Command, Response } from './types.js';
/**
 * Execute a command on the iOS manager
 */
export declare function executeIOSCommand(command: Command, manager: IOSManager): Promise<Response>;
//# sourceMappingURL=ios-actions.d.ts.map