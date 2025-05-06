type ColorFunc = (str: string | number) => string;

export type Template = {
	value: string;
	label: string;
	customCommands?: string[];
	extraEnvVarsRequired?: {value: string; label: string}[];
};

export type ApiKey = {
	apiKey: string;
	name: string;
};
