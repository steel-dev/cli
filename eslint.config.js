import js from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import pluginReact from 'eslint-plugin-react';
// import json from '@eslint/json';
// import markdown from '@eslint/markdown';
import css from '@eslint/css';
import {defineConfig, globalIgnores} from 'eslint/config';

export default defineConfig([
	globalIgnores(['dist', 'node_modules', 'examples']),
	{
		files: ['**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
		plugins: {js},
		extends: ['js/recommended'],
	},
	{
		files: ['**/*.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
		languageOptions: {globals: globals.node},
	},
	tseslint.configs.recommended,
	pluginReact.configs.flat.recommended,
	// {
	// 	files: ['**/*.json'],
	// 	plugins: {json},
	// 	language: 'json/json',
	// 	extends: ['json/recommended'],
	// },
	// {
	// 	files: ['**/*.jsonc'],
	// 	plugins: {json},
	// 	language: 'json/jsonc',
	// 	extends: ['json/recommended'],
	// },
	// {
	// 	files: ['**/*.json5'],
	// 	plugins: {json},
	// 	language: 'json/json5',
	// 	extends: ['json/recommended'],
	// },
	// {
	// 	files: ['**/*.md'],
	// 	plugins: {markdown},
	// 	language: 'markdown/gfm',
	// 	extends: ['markdown/recommended'],
	// },
	{
		files: ['**/*.css'],
		plugins: {css},
		language: 'css/css',
		extends: ['css/recommended'],
	},
]);
