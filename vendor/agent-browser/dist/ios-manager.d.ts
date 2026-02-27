/**
 * iOS Simulator Manager - Manages iOS Simulator and Safari automation via Appium.
 *
 * This provides 1:1 command parity with BrowserManager for iOS Safari.
 */
export interface IOSRefMap {
    [ref: string]: {
        selector: string;
        role?: string;
        name?: string;
        xpath?: string;
    };
}
export interface IOSEnhancedSnapshot {
    tree: string;
    refs: IOSRefMap;
}
interface IOSDeviceInfo {
    name: string;
    udid: string;
    state: string;
    runtime: string;
    isAvailable: boolean;
    isRealDevice?: boolean;
}
/**
 * Manages iOS Simulator and Safari automation via Appium
 */
export declare class IOSManager {
    private simctl;
    private browser;
    private appiumProcess;
    private deviceUdid;
    private deviceName;
    private consoleMessages;
    private refMap;
    private lastSnapshot;
    private refCounter;
    private static readonly APPIUM_PORT;
    private static readonly APPIUM_HOST;
    constructor();
    /**
     * Check if browser is launched
     */
    isLaunched(): boolean;
    /**
     * List connected real iOS devices
     */
    private listRealDevices;
    /**
     * List available iOS simulators
     */
    listDevices(): Promise<IOSDeviceInfo[]>;
    /**
     * List all devices (simulators + real devices)
     */
    listAllDevices(): Promise<IOSDeviceInfo[]>;
    /**
     * Find the best default device (most recent iPhone)
     */
    private findDefaultDevice;
    /**
     * Find device by name or UDID (searches both simulators and real devices)
     */
    private findDevice;
    /**
     * Check if Appium is installed
     */
    private checkAppiumInstalled;
    /**
     * Check if Appium server is already running
     */
    private isAppiumRunning;
    /**
     * Start Appium server if not already running
     */
    private startAppiumServer;
    /**
     * Boot the iOS simulator
     */
    private bootSimulator;
    /**
     * Launch iOS Safari via Appium
     */
    launch(options?: {
        device?: string;
        udid?: string;
        headless?: boolean;
    }): Promise<void>;
    /**
     * Navigate to URL
     */
    navigate(url: string): Promise<{
        url: string;
        title: string;
    }>;
    /**
     * Get current URL
     */
    getUrl(): Promise<string>;
    /**
     * Get page title
     */
    getTitle(): Promise<string>;
    /**
     * Click/tap an element
     */
    click(selector: string): Promise<void>;
    /**
     * Alias for click (semantic clarity for touch)
     */
    tap(selector: string): Promise<void>;
    /**
     * Type text into an element
     */
    type(selector: string, text: string, options?: {
        delay?: number;
        clear?: boolean;
    }): Promise<void>;
    /**
     * Fill an element (clear first, then type)
     */
    fill(selector: string, value: string): Promise<void>;
    /**
     * Get element by selector or ref
     */
    private getElement;
    /**
     * Get ref data from ref string
     */
    private getRefData;
    /**
     * Take a screenshot
     */
    screenshot(options?: {
        path?: string;
        fullPage?: boolean;
    }): Promise<{
        path?: string;
        base64?: string;
    }>;
    /**
     * Get page snapshot with refs
     */
    getSnapshot(options?: {
        interactive?: boolean;
    }): Promise<IOSEnhancedSnapshot>;
    /**
     * Get cached ref map
     */
    getRefMap(): IOSRefMap;
    /**
     * Scroll the page
     */
    scroll(options?: {
        selector?: string;
        x?: number;
        y?: number;
        direction?: 'up' | 'down' | 'left' | 'right';
        amount?: number;
    }): Promise<void>;
    /**
     * Swipe gesture (iOS-specific)
     */
    swipe(direction: 'up' | 'down' | 'left' | 'right', options?: {
        distance?: number;
    }): Promise<void>;
    /**
     * Execute JavaScript
     */
    evaluate<T = unknown>(script: string, ...args: unknown[]): Promise<T>;
    /**
     * Wait for element
     */
    wait(options: {
        selector?: string;
        timeout?: number;
        state?: 'attached' | 'detached' | 'visible' | 'hidden';
    }): Promise<void>;
    /**
     * Press a key
     */
    press(key: string): Promise<void>;
    /**
     * Hover over element (limited on touch - just scrolls into view)
     */
    hover(selector: string): Promise<void>;
    /**
     * Get page content (HTML)
     */
    getContent(selector?: string): Promise<string>;
    /**
     * Get text content of element
     */
    getText(selector: string): Promise<string>;
    /**
     * Get attribute value
     */
    getAttribute(selector: string, attribute: string): Promise<string | null>;
    /**
     * Check if element is visible
     */
    isVisible(selector: string): Promise<boolean>;
    /**
     * Check if element is enabled
     */
    isEnabled(selector: string): Promise<boolean>;
    /**
     * Navigate back
     */
    goBack(): Promise<void>;
    /**
     * Navigate forward
     */
    goForward(): Promise<void>;
    /**
     * Reload page
     */
    reload(): Promise<void>;
    /**
     * Select option(s) from dropdown
     */
    select(selector: string, values: string | string[]): Promise<void>;
    /**
     * Check a checkbox
     */
    check(selector: string): Promise<void>;
    /**
     * Uncheck a checkbox
     */
    uncheck(selector: string): Promise<void>;
    /**
     * Focus an element
     */
    focus(selector: string): Promise<void>;
    /**
     * Clear input field
     */
    clear(selector: string): Promise<void>;
    /**
     * Get element count
     */
    count(selector: string): Promise<number>;
    /**
     * Get bounding box
     */
    getBoundingBox(selector: string): Promise<{
        x: number;
        y: number;
        width: number;
        height: number;
    } | null>;
    /**
     * Get device info
     */
    getDeviceInfo(): {
        name: string;
        udid: string;
    } | null;
    /**
     * Close browser and cleanup
     */
    close(): Promise<void>;
}
export {};
//# sourceMappingURL=ios-manager.d.ts.map