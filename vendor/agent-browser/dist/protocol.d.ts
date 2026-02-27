import type { Command, Response } from './types.js';
export type ParseResult = {
    success: true;
    command: Command;
} | {
    success: false;
    error: string;
    id?: string;
};
/**
 * Parse a JSON string into a validated command
 */
export declare function parseCommand(input: string): ParseResult;
/**
 * Create a success response
 */
export declare function successResponse<T>(id: string, data: T): Response<T>;
/**
 * Create an error response
 */
export declare function errorResponse(id: string, error: string): Response;
/**
 * Serialize a response to JSON string
 */
export declare function serializeResponse(response: Response): string;
//# sourceMappingURL=protocol.d.ts.map