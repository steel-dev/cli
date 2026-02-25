import { devices, type Browser, type BrowserContext, type Page, type Frame, type Locator, type CDPSession } from 'playwright-core';
import type { LaunchCommand } from './types.js';
import { type RefMap, type EnhancedSnapshot } from './snapshot.js';
/**
 * Returns the default Playwright timeout in milliseconds for standard operations.
 * Can be overridden via the AGENT_BROWSER_DEFAULT_TIMEOUT environment variable.
 * Default is 25s, which is below the CLI's 30s IPC read timeout to ensure
 * Playwright errors are returned before the CLI gives up with EAGAIN.
 * CDP and recording contexts use a shorter fixed timeout (10s) and are not affected.
 */
export declare function getDefaultTimeout(): number;
export interface ScreencastFrame {
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
    sessionId: number;
}
export interface ScreencastOptions {
    format?: 'jpeg' | 'png';
    quality?: number;
    maxWidth?: number;
    maxHeight?: number;
    everyNthFrame?: number;
}
interface TrackedRequest {
    url: string;
    method: string;
    headers: Record<string, string>;
    timestamp: number;
    resourceType: string;
}
interface ConsoleMessage {
    type: string;
    text: string;
    timestamp: number;
}
interface PageError {
    message: string;
    timestamp: number;
}
/**
 * Manages the Playwright browser lifecycle with multiple tabs/windows
 */
export declare class BrowserManager {
    private browser;
    private cdpEndpoint;
    private isPersistentContext;
    private browserbaseSessionId;
    private browserbaseApiKey;
    private browserUseSessionId;
    private browserUseApiKey;
    private kernelSessionId;
    private kernelApiKey;
    private contexts;
    private pages;
    private activePageIndex;
    private activeFrame;
    private dialogHandler;
    private trackedRequests;
    private routes;
    private consoleMessages;
    private pageErrors;
    private isRecordingHar;
    private refMap;
    private lastSnapshot;
    private scopedHeaderRoutes;
    private colorScheme;
    /**
     * Set the persistent color scheme preference.
     * Applied automatically to all new pages and contexts.
     */
    setColorScheme(scheme: 'light' | 'dark' | 'no-preference' | null): void;
    private cdpSession;
    private screencastActive;
    private screencastSessionId;
    private frameCallback;
    private screencastFrameHandler;
    private recordingContext;
    private recordingPage;
    private recordingOutputPath;
    private recordingTempDir;
    private launchWarnings;
    /**
     * Get and clear launch warnings (e.g., decryption failures)
     */
    getAndClearWarnings(): string[];
    private static readonly MAX_PROFILE_EVENTS;
    private profilingActive;
    private profileChunks;
    private profileEventsDropped;
    private profileCompleteResolver;
    private profileDataHandler;
    private profileCompleteHandler;
    /**
     * Check if browser is launched
     */
    isLaunched(): boolean;
    /**
     * Get enhanced snapshot with refs and cache the ref map
     */
    getSnapshot(options?: {
        interactive?: boolean;
        cursor?: boolean;
        maxDepth?: number;
        compact?: boolean;
        selector?: string;
    }): Promise<EnhancedSnapshot>;
    /**
     * Get the last snapshot tree text (empty string if no snapshot has been taken)
     */
    getLastSnapshot(): string;
    /**
     * Update the stored snapshot (used by diff to keep the baseline current)
     */
    setLastSnapshot(snapshot: string): void;
    /**
     * Get the cached ref map from last snapshot
     */
    getRefMap(): RefMap;
    /**
     * Get a locator from a ref (e.g., "e1", "@e1", "ref=e1")
     * Returns null if ref doesn't exist or is invalid
     */
    getLocatorFromRef(refArg: string): Locator | null;
    /**
     * Check if a selector looks like a ref
     */
    isRef(selector: string): boolean;
    /**
     * Get locator - supports both refs and regular selectors
     */
    getLocator(selectorOrRef: string): Locator;
    /**
     * Check if the browser has any usable pages
     */
    hasPages(): boolean;
    /**
     * Ensure at least one page exists. If the browser is launched but all pages
     * were closed (stale session), creates a new page on the existing context.
     * No-op if pages already exist.
     */
    ensurePage(): Promise<void>;
    /**
     * Get the current active page, throws if not launched
     */
    getPage(): Page;
    /**
     * Get the current frame (or page's main frame if no frame is selected)
     */
    getFrame(): Frame;
    /**
     * Switch to a frame by selector, name, or URL
     */
    switchToFrame(options: {
        selector?: string;
        name?: string;
        url?: string;
    }): Promise<void>;
    /**
     * Switch back to main frame
     */
    switchToMainFrame(): void;
    /**
     * Set up dialog handler
     */
    setDialogHandler(response: 'accept' | 'dismiss', promptText?: string): void;
    /**
     * Clear dialog handler
     */
    clearDialogHandler(): void;
    /**
     * Start tracking requests
     */
    startRequestTracking(): void;
    /**
     * Get tracked requests
     */
    getRequests(filter?: string): TrackedRequest[];
    /**
     * Clear tracked requests
     */
    clearRequests(): void;
    /**
     * Add a route to intercept requests
     */
    addRoute(url: string, options: {
        response?: {
            status?: number;
            body?: string;
            contentType?: string;
            headers?: Record<string, string>;
        };
        abort?: boolean;
    }): Promise<void>;
    /**
     * Remove a route
     */
    removeRoute(url?: string): Promise<void>;
    /**
     * Set geolocation
     */
    setGeolocation(latitude: number, longitude: number, accuracy?: number): Promise<void>;
    /**
     * Set permissions
     */
    setPermissions(permissions: string[], grant: boolean): Promise<void>;
    /**
     * Set viewport
     */
    setViewport(width: number, height: number): Promise<void>;
    /**
     * Set device scale factor (devicePixelRatio) via CDP
     * This sets window.devicePixelRatio which affects how the page renders and responds to media queries
     *
     * Note: When using CDP to set deviceScaleFactor, screenshots will be at logical pixel dimensions
     * (viewport size), not physical pixel dimensions (viewport × scale). This is a Playwright limitation
     * when using CDP emulation on existing contexts. For true HiDPI screenshots with physical pixels,
     * deviceScaleFactor must be set at context creation time.
     *
     * Must be called after setViewport to work correctly
     */
    setDeviceScaleFactor(deviceScaleFactor: number, width: number, height: number, mobile?: boolean): Promise<void>;
    /**
     * Clear device metrics override to restore default devicePixelRatio
     */
    clearDeviceMetricsOverride(): Promise<void>;
    /**
     * Get device descriptor
     */
    getDevice(deviceName: string): (typeof devices)[keyof typeof devices] | undefined;
    /**
     * List available devices
     */
    listDevices(): string[];
    /**
     * Start console message tracking
     */
    startConsoleTracking(): void;
    /**
     * Get console messages
     */
    getConsoleMessages(): ConsoleMessage[];
    /**
     * Clear console messages
     */
    clearConsoleMessages(): void;
    /**
     * Start error tracking
     */
    startErrorTracking(): void;
    /**
     * Get page errors
     */
    getPageErrors(): PageError[];
    /**
     * Clear page errors
     */
    clearPageErrors(): void;
    /**
     * Start HAR recording
     */
    startHarRecording(): Promise<void>;
    /**
     * Check if HAR recording
     */
    isHarRecording(): boolean;
    /**
     * Set offline mode
     */
    setOffline(offline: boolean): Promise<void>;
    /**
     * Set extra HTTP headers (global - all requests)
     */
    setExtraHeaders(headers: Record<string, string>): Promise<void>;
    /**
     * Set scoped HTTP headers (only for requests matching the origin)
     * Uses route interception to add headers only to matching requests
     */
    setScopedHeaders(origin: string, headers: Record<string, string>): Promise<void>;
    /**
     * Clear scoped headers for an origin (or all if no origin specified)
     */
    clearScopedHeaders(origin?: string): Promise<void>;
    /**
     * Start tracing
     */
    startTracing(options: {
        screenshots?: boolean;
        snapshots?: boolean;
    }): Promise<void>;
    /**
     * Stop tracing and save
     */
    stopTracing(path?: string): Promise<void>;
    /**
     * Get the current browser context (first context)
     */
    getContext(): BrowserContext | null;
    /**
     * Save storage state (cookies, localStorage, etc.)
     */
    saveStorageState(path: string): Promise<void>;
    /**
     * Get all pages
     */
    getPages(): Page[];
    /**
     * Get current page index
     */
    getActiveIndex(): number;
    /**
     * Get the current browser instance
     */
    getBrowser(): Browser | null;
    /**
     * Check if an existing CDP connection is still alive
     * by verifying we can access browser contexts and that at least one has pages
     */
    private isCdpConnectionAlive;
    /**
     * Check if CDP connection needs to be re-established
     */
    private needsCdpReconnect;
    /**
     * Close a Browserbase session via API
     */
    private closeBrowserbaseSession;
    /**
     * Close a Browser Use session via API
     */
    private closeBrowserUseSession;
    /**
     * Close a Kernel session via API
     */
    private closeKernelSession;
    /**
     * Connect to Browserbase remote browser via CDP.
     * Requires BROWSERBASE_API_KEY and BROWSERBASE_PROJECT_ID environment variables.
     */
    private connectToBrowserbase;
    /**
     * Find or create a Kernel profile by name.
     * Returns the profile object if successful.
     */
    private findOrCreateKernelProfile;
    /**
     * Connect to Kernel remote browser via CDP.
     * Requires KERNEL_API_KEY environment variable.
     */
    private connectToKernel;
    /**
     * Connect to Browser Use remote browser via CDP.
     * Requires BROWSER_USE_API_KEY environment variable.
     */
    private connectToBrowserUse;
    /**
     * Launch the browser with the specified options
     * If already launched, this is a no-op (browser stays open)
     */
    launch(options: LaunchCommand): Promise<void>;
    /**
     * Connect to a running browser via CDP (Chrome DevTools Protocol)
     * @param cdpEndpoint Either a port number (as string) or a full WebSocket URL (ws:// or wss://)
     */
    private connectViaCDP;
    /**
     * Get Chrome's default user data directory paths for the current platform.
     * Returns an array of candidate paths to check (stable, then beta/canary).
     */
    private getChromeUserDataDirs;
    /**
     * Try to read the DevToolsActivePort file from a Chrome user data directory.
     * Returns { port, wsPath } if found, or null if not available.
     */
    private readDevToolsActivePort;
    /**
     * Try to discover a Chrome CDP endpoint by querying an HTTP debug port.
     * Returns the WebSocket debugger URL if available.
     */
    private probeDebugPort;
    /**
     * Auto-discover and connect to a running Chrome/Chromium instance.
     *
     * Discovery strategy:
     * 1. Read DevToolsActivePort from Chrome's default user data directories
     * 2. If found, connect using the port and WebSocket path from that file
     * 3. If not found, probe common debugging ports (9222, 9229)
     * 4. If a port responds, connect via CDP
     */
    private autoConnectViaCDP;
    /**
     * Set up console, error, and close tracking for a page
     */
    private setupPageTracking;
    /**
     * Set up tracking for new pages in a context (for CDP connections and popups/new tabs)
     * This handles pages created externally (e.g., via target="_blank" links, window.open)
     */
    private setupContextTracking;
    /**
     * Create a new tab in the current context
     */
    newTab(): Promise<{
        index: number;
        total: number;
    }>;
    /**
     * Create a new window (new context)
     */
    newWindow(viewport?: {
        width: number;
        height: number;
    } | null): Promise<{
        index: number;
        total: number;
    }>;
    /**
     * Invalidate the current CDP session (must be called before switching pages)
     * This ensures screencast and input injection work correctly after tab switch
     */
    private invalidateCDPSession;
    /**
     * Switch to a specific tab/page by index
     */
    switchTo(index: number): Promise<{
        index: number;
        url: string;
        title: string;
    }>;
    /**
     * Close a specific tab/page
     */
    closeTab(index?: number): Promise<{
        closed: number;
        remaining: number;
    }>;
    /**
     * List all tabs with their info
     */
    listTabs(): Promise<Array<{
        index: number;
        url: string;
        title: string;
        active: boolean;
    }>>;
    /**
     * Get or create a CDP session for the current page
     * Only works with Chromium-based browsers
     */
    getCDPSession(): Promise<CDPSession>;
    /**
     * Check if screencast is currently active
     */
    isScreencasting(): boolean;
    /**
     * Start screencast - streams viewport frames via CDP
     * @param callback Function called for each frame
     * @param options Screencast options
     */
    startScreencast(callback: (frame: ScreencastFrame) => void, options?: ScreencastOptions): Promise<void>;
    /**
     * Stop screencast
     */
    stopScreencast(): Promise<void>;
    /**
     * Check if profiling is currently active
     */
    isProfilingActive(): boolean;
    /**
     * Start CDP profiling (Tracing)
     */
    startProfiling(options?: {
        categories?: string[];
    }): Promise<void>;
    /**
     * Stop CDP profiling and save to file
     */
    stopProfiling(outputPath: string): Promise<{
        path: string;
        eventCount: number;
    }>;
    /**
     * Inject a mouse event via CDP
     */
    injectMouseEvent(params: {
        type: 'mousePressed' | 'mouseReleased' | 'mouseMoved' | 'mouseWheel';
        x: number;
        y: number;
        button?: 'left' | 'right' | 'middle' | 'none';
        clickCount?: number;
        deltaX?: number;
        deltaY?: number;
        modifiers?: number;
    }): Promise<void>;
    /**
     * Inject a keyboard event via CDP
     */
    injectKeyboardEvent(params: {
        type: 'keyDown' | 'keyUp' | 'char';
        key?: string;
        code?: string;
        text?: string;
        modifiers?: number;
    }): Promise<void>;
    /**
     * Inject touch event via CDP (for mobile emulation)
     */
    injectTouchEvent(params: {
        type: 'touchStart' | 'touchEnd' | 'touchMove' | 'touchCancel';
        touchPoints: Array<{
            x: number;
            y: number;
            id?: number;
        }>;
        modifiers?: number;
    }): Promise<void>;
    /**
     * Check if video recording is currently active
     */
    isRecording(): boolean;
    /**
     * Start recording to a video file using Playwright's native video recording.
     * Creates a fresh browser context with video recording enabled.
     * Automatically captures current URL and transfers cookies/storage if no URL provided.
     *
     * @param outputPath - Path to the output video file (will be .webm)
     * @param url - Optional URL to navigate to (defaults to current page URL)
     */
    startRecording(outputPath: string, url?: string): Promise<void>;
    /**
     * Stop recording and save the video file
     * @returns Recording result with path
     */
    stopRecording(): Promise<{
        path: string;
        frames: number;
        error?: string;
    }>;
    /**
     * Restart recording - stops current recording (if any) and starts a new one.
     * Convenience method that combines stopRecording and startRecording.
     *
     * @param outputPath - Path to the output video file (must be .webm)
     * @param url - Optional URL to navigate to (defaults to current page URL)
     * @returns Result from stopping the previous recording (if any)
     */
    restartRecording(outputPath: string, url?: string): Promise<{
        previousPath?: string;
        stopped: boolean;
    }>;
    /**
     * Close the browser and clean up
     */
    close(): Promise<void>;
}
export {};
//# sourceMappingURL=browser.d.ts.map