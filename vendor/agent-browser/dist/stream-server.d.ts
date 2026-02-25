import type { BrowserManager } from './browser.js';
/**
 * Check whether a WebSocket connection origin should be allowed.
 * Allows: no origin (CLI tools), file:// origins, and localhost/loopback origins.
 * Rejects: all other origins (prevents malicious web pages from connecting).
 */
export declare function isAllowedOrigin(origin: string | undefined): boolean;
export interface FrameMessage {
    type: 'frame';
    data: string;
    metadata: {
        offsetTop: number;
        pageScaleFactor: number;
        deviceWidth: number;
        deviceHeight: number;
        scrollOffsetX: number;
        scrollOffsetY: number;
        timestamp?: number;
    };
}
export interface InputMouseMessage {
    type: 'input_mouse';
    eventType: 'mousePressed' | 'mouseReleased' | 'mouseMoved' | 'mouseWheel';
    x: number;
    y: number;
    button?: 'left' | 'right' | 'middle' | 'none';
    clickCount?: number;
    deltaX?: number;
    deltaY?: number;
    modifiers?: number;
}
export interface InputKeyboardMessage {
    type: 'input_keyboard';
    eventType: 'keyDown' | 'keyUp' | 'char';
    key?: string;
    code?: string;
    text?: string;
    modifiers?: number;
}
export interface InputTouchMessage {
    type: 'input_touch';
    eventType: 'touchStart' | 'touchEnd' | 'touchMove' | 'touchCancel';
    touchPoints: Array<{
        x: number;
        y: number;
        id?: number;
    }>;
    modifiers?: number;
}
export interface StatusMessage {
    type: 'status';
    connected: boolean;
    screencasting: boolean;
    viewportWidth?: number;
    viewportHeight?: number;
}
export interface ErrorMessage {
    type: 'error';
    message: string;
}
export type StreamMessage = FrameMessage | InputMouseMessage | InputKeyboardMessage | InputTouchMessage | StatusMessage | ErrorMessage;
/**
 * WebSocket server for streaming browser viewport and receiving input
 */
export declare class StreamServer {
    private wss;
    private clients;
    private browser;
    private port;
    private isScreencasting;
    constructor(browser: BrowserManager, port?: number);
    /**
     * Start the WebSocket server
     */
    start(): Promise<void>;
    /**
     * Stop the WebSocket server
     */
    stop(): Promise<void>;
    /**
     * Handle a new WebSocket connection
     */
    private handleConnection;
    /**
     * Handle incoming messages from clients
     */
    private handleMessage;
    /**
     * Broadcast a frame to all connected clients
     */
    private broadcastFrame;
    /**
     * Send status to a client
     */
    private sendStatus;
    /**
     * Send an error to a client
     */
    private sendError;
    /**
     * Start screencasting
     */
    private startScreencast;
    /**
     * Stop screencasting
     */
    private stopScreencast;
    /**
     * Get the port the server is running on
     */
    getPort(): number;
    /**
     * Get the number of connected clients
     */
    getClientCount(): number;
}
//# sourceMappingURL=stream-server.d.ts.map