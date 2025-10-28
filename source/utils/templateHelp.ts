import zod from 'zod';
import {option} from 'pastel';
import {TEMPLATES} from './constants.js';
import type {Template} from './types.js';

/**
 * Convert environment variable name to CLI option name
 * STEEL_API_KEY -> steel_api_key
 */
function envVarToOptionName(envVar: string): string {
	return envVar.toLowerCase();
}

/**
 * Get template by alias/value
 */
export function getTemplate(identifier: string): Template | undefined {
	return TEMPLATES.find(
		t =>
			t.alias === identifier ||
			t.value === identifier ||
			t.label === identifier,
	);
}

/**
 * Check if command path is a template-specific help request
 */
export function isTemplateCommand(
	commandPath: string,
): {command: string; template: string} | null {
	const parts = commandPath.split(' ');
	if (parts.length === 2 && (parts[0] === 'run' || parts[0] === 'forge')) {
		const template = getTemplate(parts[1]);
		if (template) {
			return {command: parts[0], template: parts[1]};
		}
	}
	return null;
}

/**
 * Check if an env var is redundant with existing base options
 */
function isRedundantEnvVar(
	envVarName: string,
	command: 'run' | 'forge',
	template: Template,
): boolean {
	const alwaysRedundant = [
		'STEEL_API_KEY', // covered by api_key
		'STEEL_API_URL', // covered by api_url
	];

	// These are conditionally redundant based on template support and command type
	// const conditionallyRedundant = [
	// 	'OPENAI_API_KEY', // covered by openai_key only if template supports OpenAI
	// 	'TASK', // covered by task only if template supports task AND it's a run command
	// 	'ANTHROPIC_API_KEY', // covered by anthropic_key only if template supports anthropic
	// ];

	if (alwaysRedundant.includes(envVarName)) {
		return true;
	}

	if (envVarName === 'OPENAI_API_KEY' && templateSupports(template, 'openai')) {
		return true;
	}

	if (envVarName === 'TASK') {
		// TASK is always redundant for forge commands (not supported)
		// For run commands, it's redundant only if we're including the task base option
		return (
			command === 'forge' ||
			(command === 'run' && templateSupports(template, 'task'))
		);
	}

	if (
		envVarName === 'ANTHROPIC_API_KEY' &&
		templateSupports(template, 'anthropic')
	) {
		return true;
	}

	if (envVarName === 'GEMINI_API_KEY' && templateSupports(template, 'gemini')) {
		return true;
	}

	return false;
}

/**
 * Check if template supports a specific capability
 */
function templateSupports(template: Template, capability: string): boolean {
	const envVarMap = {
		task: 'TASK',
		openai: 'OPENAI_API_KEY',
		anthropic: 'ANTHROPIC_API_KEY',
		gemini: 'GEMINI_API_KEY',
	};

	const envVarName = envVarMap[capability];
	if (!envVarName) return false;

	return template.env.some(env => env.value === envVarName);
}

/**
 * Create a virtual command module for template-specific help
 */
export function createTemplateCommandModule(
	command: 'run' | 'forge',
	templateAlias: string,
) {
	const template = getTemplate(templateAlias);
	if (!template) {
		throw new Error(`Template "${templateAlias}" not found`);
	}

	// Common base options for all templates
	const baseOptions: Record<string, zod.ZodType> = {
		api_url: zod
			.string()
			.describe(
				option({
					description: 'API URL for Steel API',
					alias: 'a',
				}),
			)
			.optional(),
		api_key: zod
			.string()
			.describe(
				option({
					description: 'API Key for Steel API',
				}),
			)
			.optional(),
		skip_auth: zod.boolean().describe('Skip authentication').optional(),
		help: zod
			.boolean()
			.describe(option({description: 'Show help', alias: 'h'}))
			.optional(),
	};

	// Add command-specific options
	if (command === 'run') {
		// Always include view option for run commands
		baseOptions.view = zod
			.boolean()
			.describe(
				option({
					description: 'Open live session viewer',
					alias: 'o',
				}),
			)
			.optional();

		// Only include task option if template supports it
		if (templateSupports(template, 'task')) {
			baseOptions.task = zod
				.string()
				.describe(
					option({
						description: 'Task to run',
						alias: 't',
					}),
				)
				.optional();
		}
	}

	if (command === 'forge') {
		// Always include name option for forge commands
		baseOptions.name = zod
			.string()
			.describe(
				option({
					description: 'Name of the project',
					alias: 'n',
				}),
			)
			.optional();
	}

	// Only include openai_key if template supports OpenAI
	if (templateSupports(template, 'openai')) {
		baseOptions.openai_key = zod
			.string()
			.describe(option({description: 'API Key for OpenAI'}))
			.optional();
	}

	if (templateSupports(template, 'anthropic')) {
		baseOptions.anthropic_key = zod
			.string()
			.describe(option({description: 'API Key for Anthropic'}))
			.optional();
	}

	if (templateSupports(template, 'gemini')) {
		baseOptions.gemini_key = zod
			.string()
			.describe(option({description: 'API Key for Gemini'}))
			.optional();
	}

	// Template-specific options from env vars (excluding redundant ones)
	const templateOptions: Record<string, zod.ZodType> = {};
	template.env
		.filter(envVar => !isRedundantEnvVar(envVar.value, command, template))
		.forEach(envVar => {
			const optionName = envVarToOptionName(envVar.value);
			templateOptions[optionName] = zod
				.string()
				.describe(
					option({
						description: `${envVar.label}${envVar.required ? ' (required)' : ''}`,
					}),
				)
				.optional();
		});

	// Combine base options with template-specific options
	const combinedOptions = {...baseOptions, ...templateOptions};

	return {
		description: `Run template automation for: ${template.label}`,
		options: zod.object(combinedOptions),
		args: undefined, // Templates don't take additional arguments
		argsLabels: [],
		template, // Include template info for display
	};
}
