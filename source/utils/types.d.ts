type ColorFunc = (str: string | number) => string;

export type Template = {
	value: string;
	label: string;
	alias: string;
	command: string;
	/**
	 * Primary language used by the starter (TS, JS, PY, etc.).
	 */
	language?: string;
	/**
	 * High-level grouping such as "Browser", "AI", "Files", etc.
	 */
	category?: string;
	/**
	 * Accent color for UI display (blue, yellow, orange, purple, green, etc.)
	 */
	accentColor?: string;
	/**
	 * Optional group ID that this template belongs to
	 */
	groupId?: string;
	depCommands?: (options: {depsDir: string}) => string[];
	runCommand?: (options: {depsDir: string}) => string;
	/**
	 * Simple run commands for user display (without complex paths/NODE_PATH setup)
	 */
	displayRunCommands?: (options: {
		directory: string;
		packageManager?: string;
	}) => string[];
	env?: {value: string; label: string; required?: boolean}[];
};

export type ApiKey = {
	apiKey: string;
	name: string;
};

export type LoadingState = 'loading';
export type OtherStates = 'pending' | 'success' | 'warning' | 'error';

export type CodeSections = {
	imports: string[];
	exports: string[];
	body: string[];
};

export type TemplateOptions = {
	depsDir: string;
};
