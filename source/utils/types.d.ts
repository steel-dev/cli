type ColorFunc = (str: string | number) => string;

export type Template = {
	value: string;
	label: string;
	alias: string;
	/**
	 * Primary language used by the starter (TS, JS, PY, etc.).
	 */
	language?: string;
	/**
	 * High-level grouping such as "Browser", "AI", "Files", etc.
	 */
	category?: string;
	depCommands?: (options: {depsDir: string}) => string[];
	runCommand?: (options: {depsDir: string}) => string;
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
