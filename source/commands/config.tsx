#!/usr/bin/env node

import React from 'react';
import Callout from '../components/callout.js';
import {getApiKey, getSettings} from '../utils/session.js';

export const description = 'Display information about the current session';

export default function Info() {
	const apiKey = getApiKey();
	const settings = getSettings();

	// Build configuration info text
	let configInfo = '';

	if (apiKey) {
		Object.keys(apiKey).forEach(key => {
			if (key === 'apiKey') {
				configInfo += `${key}: ${apiKey[key as keyof typeof apiKey].substring(0, 7) + '...'}\n`;
			} else {
				configInfo += `${key}: ${apiKey[key as keyof typeof apiKey]}\n`;
			}
		});
	}

	if (settings) {
		Object.keys(settings).forEach(key => {
			configInfo += `${key}: ${settings[key as keyof typeof settings]}\n`;
		});
	}

	return apiKey ? (
		<Callout variant="info" title="Current Configuration">
			{configInfo.trim()}
		</Callout>
	) : (
		<Callout variant="warning" title="Authentication Required">
			You are not logged in. Please run `steel login` to authenticate.
		</Callout>
	);
}
