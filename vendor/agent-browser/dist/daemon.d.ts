import * as net from 'net';
/**
 * Backpressure-aware socket write.
 * If the kernel buffer is full (socket.write returns false),
 * waits for the 'drain' event before resolving.
 */
export declare function safeWrite(socket: net.Socket, payload: string): Promise<void>;
/**
 * Set the current session
 */
export declare function setSession(session: string): void;
/**
 * Get the current session
 */
export declare function getSession(): string;
/**
 * Get the base directory for socket/pid files.
 * Priority: AGENT_BROWSER_SOCKET_DIR > XDG_RUNTIME_DIR > ~/.agent-browser > tmpdir
 */
export declare function getAppDir(): string;
export declare function getSocketDir(): string;
/**
 * Get the socket path for the current session (Unix) or port (Windows)
 */
export declare function getSocketPath(session?: string): string;
/**
 * Get the port file path for Windows (stores the port number)
 */
export declare function getPortFile(session?: string): string;
/**
 * Get the PID file path for the current session
 */
export declare function getPidFile(session?: string): string;
/**
 * Check if daemon is running for the current session
 */
export declare function isDaemonRunning(session?: string): boolean;
/**
 * Get connection info for the current session
 * Returns { type: 'unix', path: string } or { type: 'tcp', port: number }
 */
export declare function getConnectionInfo(session?: string): {
    type: 'unix';
    path: string;
} | {
    type: 'tcp';
    port: number;
};
/**
 * Clean up socket and PID file for the current session
 */
export declare function cleanupSocket(session?: string): void;
/**
 * Get the stream port file path
 */
export declare function getStreamPortFile(session?: string): string;
/**
 * Start the daemon server
 * @param options.streamPort Port for WebSocket stream server (0 to disable)
 * @param options.provider Provider type ('ios' for iOS Simulator, undefined for desktop)
 */
export declare function startDaemon(options?: {
    streamPort?: number;
    provider?: string;
}): Promise<void>;
//# sourceMappingURL=daemon.d.ts.map