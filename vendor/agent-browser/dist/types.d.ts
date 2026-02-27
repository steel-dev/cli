import type { Page, Browser, BrowserContext } from 'playwright-core';
export interface BaseCommand {
    id: string;
    action: string;
}
export interface LaunchCommand extends BaseCommand {
    action: 'launch';
    headless?: boolean;
    viewport?: {
        width: number;
        height: number;
    } | null;
    browser?: 'chromium' | 'firefox' | 'webkit';
    headers?: Record<string, string>;
    executablePath?: string;
    cdpPort?: number;
    cdpUrl?: string;
    autoConnect?: boolean;
    extensions?: string[];
    profile?: string;
    storageState?: string;
    proxy?: {
        server: string;
        bypass?: string;
        username?: string;
        password?: string;
    };
    args?: string[];
    userAgent?: string;
    provider?: string;
    ignoreHTTPSErrors?: boolean;
    allowFileAccess?: boolean;
    colorScheme?: 'light' | 'dark' | 'no-preference';
    autoStateFilePath?: string;
}
export interface NavigateCommand extends BaseCommand {
    action: 'navigate';
    url: string;
    waitUntil?: 'load' | 'domcontentloaded' | 'networkidle';
    headers?: Record<string, string>;
}
export interface ClickCommand extends BaseCommand {
    action: 'click';
    selector: string;
    button?: 'left' | 'right' | 'middle';
    clickCount?: number;
    delay?: number;
    newTab?: boolean;
}
export interface TypeCommand extends BaseCommand {
    action: 'type';
    selector: string;
    text: string;
    delay?: number;
    clear?: boolean;
}
export interface FillCommand extends BaseCommand {
    action: 'fill';
    selector: string;
    value: string;
}
export interface CheckCommand extends BaseCommand {
    action: 'check';
    selector: string;
}
export interface UncheckCommand extends BaseCommand {
    action: 'uncheck';
    selector: string;
}
export interface UploadCommand extends BaseCommand {
    action: 'upload';
    selector: string;
    files: string | string[];
}
export interface DoubleClickCommand extends BaseCommand {
    action: 'dblclick';
    selector: string;
}
export interface FocusCommand extends BaseCommand {
    action: 'focus';
    selector: string;
}
export interface DragCommand extends BaseCommand {
    action: 'drag';
    source: string;
    target: string;
}
export interface FrameCommand extends BaseCommand {
    action: 'frame';
    selector?: string;
    name?: string;
    url?: string;
}
export interface MainFrameCommand extends BaseCommand {
    action: 'mainframe';
}
export interface GetByRoleCommand extends BaseCommand {
    action: 'getbyrole';
    role: string;
    name?: string;
    exact?: boolean;
    subaction: 'click' | 'fill' | 'check' | 'hover';
    value?: string;
}
export interface GetByTextCommand extends BaseCommand {
    action: 'getbytext';
    text: string;
    exact?: boolean;
    subaction: 'click' | 'hover';
}
export interface GetByLabelCommand extends BaseCommand {
    action: 'getbylabel';
    label: string;
    exact?: boolean;
    subaction: 'click' | 'fill' | 'check';
    value?: string;
}
export interface GetByPlaceholderCommand extends BaseCommand {
    action: 'getbyplaceholder';
    placeholder: string;
    exact?: boolean;
    subaction: 'click' | 'fill';
    value?: string;
}
export interface CookiesGetCommand extends BaseCommand {
    action: 'cookies_get';
    urls?: string[];
}
export interface CookiesSetCommand extends BaseCommand {
    action: 'cookies_set';
    cookies: Array<{
        name: string;
        value: string;
        url?: string;
        domain?: string;
        path?: string;
        expires?: number;
        httpOnly?: boolean;
        secure?: boolean;
        sameSite?: 'Strict' | 'Lax' | 'None';
    }>;
}
export interface CookiesClearCommand extends BaseCommand {
    action: 'cookies_clear';
}
export interface StorageGetCommand extends BaseCommand {
    action: 'storage_get';
    key?: string;
    type: 'local' | 'session';
}
export interface StorageSetCommand extends BaseCommand {
    action: 'storage_set';
    key: string;
    value: string;
    type: 'local' | 'session';
}
export interface StorageClearCommand extends BaseCommand {
    action: 'storage_clear';
    type: 'local' | 'session';
}
export interface DialogCommand extends BaseCommand {
    action: 'dialog';
    response: 'accept' | 'dismiss';
    promptText?: string;
}
export interface PdfCommand extends BaseCommand {
    action: 'pdf';
    path: string;
    format?: 'Letter' | 'Legal' | 'Tabloid' | 'Ledger' | 'A0' | 'A1' | 'A2' | 'A3' | 'A4' | 'A5' | 'A6';
}
export interface RouteCommand extends BaseCommand {
    action: 'route';
    url: string;
    response?: {
        status?: number;
        body?: string;
        contentType?: string;
        headers?: Record<string, string>;
    };
    abort?: boolean;
}
export interface UnrouteCommand extends BaseCommand {
    action: 'unroute';
    url?: string;
}
export interface RequestsCommand extends BaseCommand {
    action: 'requests';
    filter?: string;
    clear?: boolean;
}
export interface DownloadCommand extends BaseCommand {
    action: 'download';
    selector: string;
    path: string;
}
export interface GeolocationCommand extends BaseCommand {
    action: 'geolocation';
    latitude: number;
    longitude: number;
    accuracy?: number;
}
export interface PermissionsCommand extends BaseCommand {
    action: 'permissions';
    permissions: string[];
    grant: boolean;
}
export interface ViewportCommand extends BaseCommand {
    action: 'viewport';
    width: number;
    height: number;
}
export interface UserAgentCommand extends BaseCommand {
    action: 'useragent';
    userAgent: string;
}
export interface DeviceCommand extends BaseCommand {
    action: 'device';
    device: string;
}
export interface BackCommand extends BaseCommand {
    action: 'back';
}
export interface ForwardCommand extends BaseCommand {
    action: 'forward';
}
export interface ReloadCommand extends BaseCommand {
    action: 'reload';
}
export interface UrlCommand extends BaseCommand {
    action: 'url';
}
export interface TitleCommand extends BaseCommand {
    action: 'title';
}
export interface GetAttributeCommand extends BaseCommand {
    action: 'getattribute';
    selector: string;
    attribute: string;
}
export interface GetTextCommand extends BaseCommand {
    action: 'gettext';
    selector: string;
}
export interface IsVisibleCommand extends BaseCommand {
    action: 'isvisible';
    selector: string;
}
export interface IsEnabledCommand extends BaseCommand {
    action: 'isenabled';
    selector: string;
}
export interface IsCheckedCommand extends BaseCommand {
    action: 'ischecked';
    selector: string;
}
export interface CountCommand extends BaseCommand {
    action: 'count';
    selector: string;
}
export interface BoundingBoxCommand extends BaseCommand {
    action: 'boundingbox';
    selector: string;
}
export interface StylesCommand extends BaseCommand {
    action: 'styles';
    selector: string;
}
export interface GetByAltTextCommand extends BaseCommand {
    action: 'getbyalttext';
    text: string;
    exact?: boolean;
    subaction: 'click' | 'hover';
}
export interface GetByTitleCommand extends BaseCommand {
    action: 'getbytitle';
    text: string;
    exact?: boolean;
    subaction: 'click' | 'hover';
}
export interface GetByTestIdCommand extends BaseCommand {
    action: 'getbytestid';
    testId: string;
    subaction: 'click' | 'fill' | 'check' | 'hover';
    value?: string;
}
export interface NthCommand extends BaseCommand {
    action: 'nth';
    selector: string;
    index: number;
    subaction: 'click' | 'fill' | 'check' | 'hover' | 'text';
    value?: string;
}
export interface WaitForUrlCommand extends BaseCommand {
    action: 'waitforurl';
    url: string;
    timeout?: number;
}
export interface WaitForLoadStateCommand extends BaseCommand {
    action: 'waitforloadstate';
    state: 'load' | 'domcontentloaded' | 'networkidle';
    timeout?: number;
}
export interface SetContentCommand extends BaseCommand {
    action: 'setcontent';
    html: string;
}
export interface TimezoneCommand extends BaseCommand {
    action: 'timezone';
    timezone: string;
}
export interface LocaleCommand extends BaseCommand {
    action: 'locale';
    locale: string;
}
export interface HttpCredentialsCommand extends BaseCommand {
    action: 'credentials';
    username: string;
    password: string;
}
export interface MouseMoveCommand extends BaseCommand {
    action: 'mousemove';
    x: number;
    y: number;
}
export interface MouseDownCommand extends BaseCommand {
    action: 'mousedown';
    button?: 'left' | 'right' | 'middle';
}
export interface MouseUpCommand extends BaseCommand {
    action: 'mouseup';
    button?: 'left' | 'right' | 'middle';
}
export interface BringToFrontCommand extends BaseCommand {
    action: 'bringtofront';
}
export interface WaitForFunctionCommand extends BaseCommand {
    action: 'waitforfunction';
    expression: string;
    timeout?: number;
}
export interface ScrollIntoViewCommand extends BaseCommand {
    action: 'scrollintoview';
    selector: string;
}
export interface AddInitScriptCommand extends BaseCommand {
    action: 'addinitscript';
    script: string;
}
export interface KeyDownCommand extends BaseCommand {
    action: 'keydown';
    key: string;
}
export interface KeyUpCommand extends BaseCommand {
    action: 'keyup';
    key: string;
}
export interface InsertTextCommand extends BaseCommand {
    action: 'inserttext';
    text: string;
}
export interface MultiSelectCommand extends BaseCommand {
    action: 'multiselect';
    selector: string;
    values: string[];
}
export interface WaitForDownloadCommand extends BaseCommand {
    action: 'waitfordownload';
    path?: string;
    timeout?: number;
}
export interface ResponseBodyCommand extends BaseCommand {
    action: 'responsebody';
    url: string;
    timeout?: number;
}
export interface ScreencastStartCommand extends BaseCommand {
    action: 'screencast_start';
    format?: 'jpeg' | 'png';
    quality?: number;
    maxWidth?: number;
    maxHeight?: number;
    everyNthFrame?: number;
}
export interface ScreencastStopCommand extends BaseCommand {
    action: 'screencast_stop';
}
export interface InputMouseCommand extends BaseCommand {
    action: 'input_mouse';
    type: 'mousePressed' | 'mouseReleased' | 'mouseMoved' | 'mouseWheel';
    x: number;
    y: number;
    button?: 'left' | 'right' | 'middle' | 'none';
    clickCount?: number;
    deltaX?: number;
    deltaY?: number;
    modifiers?: number;
}
export interface InputKeyboardCommand extends BaseCommand {
    action: 'input_keyboard';
    type: 'keyDown' | 'keyUp' | 'char';
    key?: string;
    code?: string;
    text?: string;
    modifiers?: number;
}
export interface InputTouchCommand extends BaseCommand {
    action: 'input_touch';
    type: 'touchStart' | 'touchEnd' | 'touchMove' | 'touchCancel';
    touchPoints: Array<{
        x: number;
        y: number;
        id?: number;
    }>;
    modifiers?: number;
}
export interface SwipeCommand extends BaseCommand {
    action: 'swipe';
    direction: 'up' | 'down' | 'left' | 'right';
    distance?: number;
}
export interface DeviceListCommand extends BaseCommand {
    action: 'device_list';
}
export interface VideoStartCommand extends BaseCommand {
    action: 'video_start';
    path: string;
}
export interface VideoStopCommand extends BaseCommand {
    action: 'video_stop';
}
export interface RecordingStartCommand extends BaseCommand {
    action: 'recording_start';
    path: string;
    url?: string;
}
export interface RecordingStopCommand extends BaseCommand {
    action: 'recording_stop';
}
export interface RecordingRestartCommand extends BaseCommand {
    action: 'recording_restart';
    path: string;
    url?: string;
}
export interface TraceStartCommand extends BaseCommand {
    action: 'trace_start';
    screenshots?: boolean;
    snapshots?: boolean;
}
export interface TraceStopCommand extends BaseCommand {
    action: 'trace_stop';
    path?: string;
}
/**
 * Chrome Trace Event format. All fields are optional because CDP trace event
 * shapes vary across categories and event phases -- this type is intentionally
 * loose to accept any valid trace event without data loss.
 */
export interface TraceEvent {
    cat?: string;
    name?: string;
    ph?: string;
    pid?: number;
    tid?: number;
    ts?: number;
    dur?: number;
    args?: Record<string, unknown>;
    [key: string]: unknown;
}
export interface ProfilerStartCommand extends BaseCommand {
    action: 'profiler_start';
    categories?: string[];
}
export interface ProfilerStopCommand extends BaseCommand {
    action: 'profiler_stop';
    path?: string;
}
export interface HarStartCommand extends BaseCommand {
    action: 'har_start';
}
export interface HarStopCommand extends BaseCommand {
    action: 'har_stop';
    path: string;
}
export interface StorageStateSaveCommand extends BaseCommand {
    action: 'state_save';
    path: string;
}
export interface StorageStateLoadCommand extends BaseCommand {
    action: 'state_load';
    path: string;
}
export interface StateListCommand extends BaseCommand {
    action: 'state_list';
}
export interface StateClearCommand extends BaseCommand {
    action: 'state_clear';
    sessionName?: string;
    all?: boolean;
}
export interface StateShowCommand extends BaseCommand {
    action: 'state_show';
    filename: string;
}
export interface StateCleanCommand extends BaseCommand {
    action: 'state_clean';
    days: number;
}
export interface StateRenameCommand extends BaseCommand {
    action: 'state_rename';
    oldName: string;
    newName: string;
}
export interface ConsoleCommand extends BaseCommand {
    action: 'console';
    clear?: boolean;
}
export interface ErrorsCommand extends BaseCommand {
    action: 'errors';
    clear?: boolean;
}
export interface KeyboardCommand extends BaseCommand {
    action: 'keyboard';
    subaction?: 'type' | 'press' | 'insertText';
    keys?: string;
    text?: string;
    delay?: number;
}
export interface WheelCommand extends BaseCommand {
    action: 'wheel';
    deltaX?: number;
    deltaY?: number;
    selector?: string;
}
export interface TapCommand extends BaseCommand {
    action: 'tap';
    selector: string;
}
export interface ClipboardCommand extends BaseCommand {
    action: 'clipboard';
    operation: 'copy' | 'paste' | 'read';
    text?: string;
}
export interface HighlightCommand extends BaseCommand {
    action: 'highlight';
    selector: string;
}
export interface ClearCommand extends BaseCommand {
    action: 'clear';
    selector: string;
}
export interface SelectAllCommand extends BaseCommand {
    action: 'selectall';
    selector: string;
}
export interface InnerTextCommand extends BaseCommand {
    action: 'innertext';
    selector: string;
}
export interface InnerHtmlCommand extends BaseCommand {
    action: 'innerhtml';
    selector: string;
}
export interface InputValueCommand extends BaseCommand {
    action: 'inputvalue';
    selector: string;
}
export interface SetValueCommand extends BaseCommand {
    action: 'setvalue';
    selector: string;
    value: string;
}
export interface DispatchEventCommand extends BaseCommand {
    action: 'dispatch';
    selector: string;
    event: string;
    eventInit?: Record<string, unknown>;
}
export interface EvaluateHandleCommand extends BaseCommand {
    action: 'evalhandle';
    script: string;
}
export interface ExposeFunctionCommand extends BaseCommand {
    action: 'expose';
    name: string;
}
export interface AddScriptCommand extends BaseCommand {
    action: 'addscript';
    content?: string;
    url?: string;
}
export interface AddStyleCommand extends BaseCommand {
    action: 'addstyle';
    content?: string;
    url?: string;
}
export interface EmulateMediaCommand extends BaseCommand {
    action: 'emulatemedia';
    media?: 'screen' | 'print' | null;
    colorScheme?: 'light' | 'dark' | 'no-preference' | null;
    reducedMotion?: 'reduce' | 'no-preference' | null;
    forcedColors?: 'active' | 'none' | null;
}
export interface OfflineCommand extends BaseCommand {
    action: 'offline';
    offline: boolean;
}
export interface HeadersCommand extends BaseCommand {
    action: 'headers';
    headers: Record<string, string>;
}
export interface PauseCommand extends BaseCommand {
    action: 'pause';
}
export interface PressCommand extends BaseCommand {
    action: 'press';
    key: string;
    selector?: string;
}
export interface ScreenshotCommand extends BaseCommand {
    action: 'screenshot';
    path?: string;
    fullPage?: boolean;
    selector?: string;
    format?: 'png' | 'jpeg';
    quality?: number;
    annotate?: boolean;
}
export interface SnapshotCommand extends BaseCommand {
    action: 'snapshot';
}
export interface EvaluateCommand extends BaseCommand {
    action: 'evaluate';
    script: string;
    args?: unknown[];
}
export interface WaitCommand extends BaseCommand {
    action: 'wait';
    selector?: string;
    timeout?: number;
    state?: 'attached' | 'detached' | 'visible' | 'hidden';
}
export interface ScrollCommand extends BaseCommand {
    action: 'scroll';
    selector?: string;
    x?: number;
    y?: number;
    direction?: 'up' | 'down' | 'left' | 'right';
    amount?: number;
}
export interface SelectCommand extends BaseCommand {
    action: 'select';
    selector: string;
    values: string | string[];
}
export interface HoverCommand extends BaseCommand {
    action: 'hover';
    selector: string;
}
export interface ContentCommand extends BaseCommand {
    action: 'content';
    selector?: string;
}
export interface CloseCommand extends BaseCommand {
    action: 'close';
}
export interface TabNewCommand extends BaseCommand {
    action: 'tab_new';
    url?: string;
}
export interface TabListCommand extends BaseCommand {
    action: 'tab_list';
}
export interface TabSwitchCommand extends BaseCommand {
    action: 'tab_switch';
    index: number;
}
export interface TabCloseCommand extends BaseCommand {
    action: 'tab_close';
    index?: number;
}
export interface WindowNewCommand extends BaseCommand {
    action: 'window_new';
    viewport?: {
        width: number;
        height: number;
    } | null;
}
export type Command = LaunchCommand | NavigateCommand | ClickCommand | TypeCommand | FillCommand | CheckCommand | UncheckCommand | UploadCommand | DoubleClickCommand | FocusCommand | DragCommand | FrameCommand | MainFrameCommand | GetByRoleCommand | GetByTextCommand | GetByLabelCommand | GetByPlaceholderCommand | PressCommand | ScreenshotCommand | SnapshotCommand | EvaluateCommand | WaitCommand | ScrollCommand | SelectCommand | HoverCommand | ContentCommand | CloseCommand | TabNewCommand | TabListCommand | TabSwitchCommand | TabCloseCommand | WindowNewCommand | CookiesGetCommand | CookiesSetCommand | CookiesClearCommand | StorageGetCommand | StorageSetCommand | StorageClearCommand | DialogCommand | PdfCommand | RouteCommand | UnrouteCommand | RequestsCommand | DownloadCommand | GeolocationCommand | PermissionsCommand | ViewportCommand | UserAgentCommand | DeviceCommand | BackCommand | ForwardCommand | ReloadCommand | UrlCommand | TitleCommand | GetAttributeCommand | GetTextCommand | IsVisibleCommand | IsEnabledCommand | IsCheckedCommand | CountCommand | BoundingBoxCommand | StylesCommand | VideoStartCommand | VideoStopCommand | RecordingStartCommand | RecordingStopCommand | RecordingRestartCommand | TraceStartCommand | TraceStopCommand | ProfilerStartCommand | ProfilerStopCommand | HarStartCommand | HarStopCommand | StorageStateSaveCommand | StorageStateLoadCommand | StateListCommand | StateClearCommand | StateShowCommand | StateCleanCommand | StateRenameCommand | ConsoleCommand | ErrorsCommand | KeyboardCommand | WheelCommand | TapCommand | ClipboardCommand | HighlightCommand | ClearCommand | SelectAllCommand | InnerTextCommand | InnerHtmlCommand | InputValueCommand | SetValueCommand | DispatchEventCommand | EvaluateHandleCommand | ExposeFunctionCommand | AddScriptCommand | AddStyleCommand | EmulateMediaCommand | OfflineCommand | HeadersCommand | PauseCommand | GetByAltTextCommand | GetByTitleCommand | GetByTestIdCommand | NthCommand | WaitForUrlCommand | WaitForLoadStateCommand | SetContentCommand | TimezoneCommand | LocaleCommand | HttpCredentialsCommand | MouseMoveCommand | MouseDownCommand | MouseUpCommand | BringToFrontCommand | WaitForFunctionCommand | ScrollIntoViewCommand | AddInitScriptCommand | KeyDownCommand | KeyUpCommand | InsertTextCommand | MultiSelectCommand | WaitForDownloadCommand | ResponseBodyCommand | ScreencastStartCommand | ScreencastStopCommand | InputMouseCommand | InputKeyboardCommand | InputTouchCommand | SwipeCommand | DeviceListCommand | DiffSnapshotCommand | DiffScreenshotCommand | DiffUrlCommand;
export interface DiffSnapshotCommand extends BaseCommand {
    action: 'diff_snapshot';
    baseline?: string;
    selector?: string;
    compact?: boolean;
    maxDepth?: number;
}
export interface DiffScreenshotCommand extends BaseCommand {
    action: 'diff_screenshot';
    baseline: string;
    output?: string;
    threshold?: number;
    selector?: string;
    fullPage?: boolean;
}
export interface DiffUrlCommand extends BaseCommand {
    action: 'diff_url';
    url1: string;
    url2: string;
    screenshot?: boolean;
    fullPage?: boolean;
    waitUntil?: 'load' | 'domcontentloaded' | 'networkidle';
    selector?: string;
    compact?: boolean;
    maxDepth?: number;
}
export interface SuccessResponse<T = unknown> {
    id: string;
    success: true;
    data: T;
}
export interface ErrorResponse {
    id: string;
    success: false;
    error: string;
}
export type Response<T = unknown> = SuccessResponse<T> | ErrorResponse;
export interface NavigateData {
    url: string;
    title: string;
}
export interface Annotation {
    ref: string;
    number: number;
    role: string;
    name?: string;
    box: {
        x: number;
        y: number;
        width: number;
        height: number;
    };
}
export interface ScreenshotData {
    path?: string;
    base64?: string;
    annotations?: Annotation[];
}
export interface SnapshotData {
    snapshot: string;
}
export interface EvaluateData {
    result: unknown;
}
export interface ContentData {
    html: string;
}
export interface TabInfo {
    index: number;
    url: string;
    title: string;
    active: boolean;
}
export interface TabListData {
    tabs: TabInfo[];
    active: number;
}
export interface TabNewData {
    index: number;
    total: number;
}
export interface TabSwitchData {
    index: number;
    url: string;
    title: string;
}
export interface TabCloseData {
    closed: number;
    remaining: number;
}
export interface ScreencastStartData {
    started: boolean;
    format: string;
    quality: number;
}
export interface ScreencastStopData {
    stopped: boolean;
}
export interface RecordingStartData {
    started: boolean;
    path: string;
}
export interface RecordingStopData {
    path: string;
    frames: number;
    error?: string;
}
export interface RecordingRestartData {
    started: boolean;
    path: string;
    previousPath?: string;
    stopped: boolean;
}
export interface InputEventData {
    injected: boolean;
}
export interface ElementStyleInfo {
    tag: string;
    text: string | null;
    box: {
        x: number;
        y: number;
        width: number;
        height: number;
    };
    styles: {
        fontSize: string;
        fontWeight: string;
        fontFamily: string;
        color: string;
        backgroundColor: string;
        borderRadius: string;
        border: string | null;
        boxShadow: string | null;
        padding: string;
    };
}
export interface StylesData {
    elements: ElementStyleInfo[];
}
export interface DiffSnapshotData {
    diff: string;
    additions: number;
    removals: number;
    unchanged: number;
    changed: boolean;
}
export interface DiffScreenshotData {
    diffPath: string;
    totalPixels: number;
    differentPixels: number;
    mismatchPercentage: number;
    match: boolean;
    dimensionMismatch?: boolean;
}
export interface DiffUrlData {
    snapshot: DiffSnapshotData;
    screenshot?: DiffScreenshotData;
}
export interface BrowserState {
    browser: Browser | null;
    context: BrowserContext | null;
    page: Page | null;
}
//# sourceMappingURL=types.d.ts.map