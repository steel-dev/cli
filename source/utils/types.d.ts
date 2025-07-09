type ColorFunc = (str: string | number) => string;

export type Template = {
	value: string;
	label: string;
	alias: string;
	commands?: string[];
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
