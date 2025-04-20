#!/usr/bin/env node
// import fetch from 'node-fetch';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';

// Config paths
const CONFIG_DIR = path.join(os.homedir(), '.steel-cli');
const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');

// Base API URL
const API_BASE_URL = 'https://api.yourservice.com/v1';

// Function to get the API key from config
export async function getApiKey(): Promise<string | null> {
	try {
		const configData = await fs.readFile(CONFIG_PATH, 'utf-8');
		const config = JSON.parse(configData);
		return config.apiKey || null;
	} catch (error) {
		return null;
	}
}

// API client class - same as before
export class ApiClient {
	private apiKey: string | null = null;

	constructor(apiKey?: string) {
		this.apiKey = apiKey || null;
	}

	async initialize(): Promise<boolean> {
		if (!this.apiKey) {
			this.apiKey = await getApiKey();
		}
		return !!this.apiKey;
	}

	async request<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
		// Initialize if not already done
		if (!this.apiKey) {
			const initialized = await this.initialize();
			if (!initialized) {
				throw new Error(
					'Not authenticated. Please run `your-cli-name auth` first.',
				);
			}
		}

		const url = `${API_BASE_URL}${endpoint}`;
		const headers = {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${this.apiKey}`,
			...options.headers,
		};

		const response = await fetch(url, {
			method: options.method || 'GET',
			headers,
			body: options.body ? JSON.stringify(options.body) : undefined,
		});

		if (!response.ok) {
			const text = await response.text();
			throw new Error(
				`API request failed: ${response.status} ${response.statusText}\n${text}`,
			);
		}

		return response.json() as Promise<T>;
	}

	// Helper methods for common operations
	async get<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
		return this.request<T>(endpoint, {...options, method: 'GET'});
	}

	async post<T>(
		endpoint: string,
		data: any,
		options: RequestOptions = {},
	): Promise<T> {
		return this.request<T>(endpoint, {...options, method: 'POST', body: data});
	}

	// Add your API methods here
}

// Types
interface RequestOptions {
	method?: string;
	body?: any;
	headers?: Record<string, string>;
}
