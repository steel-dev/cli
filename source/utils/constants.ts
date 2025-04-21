import path from 'path';
import os from 'os';

export const TARGET_SITE = 'https://app.steel.dev/sign-in';
export const TARGET_API_PATH = 'https://api.steel.dev/v1/api-keys/';
export const API_PATH = 'https://api.steel.dev/v1';
export const CONFIG_DIR = path.join(os.homedir(), '.steel-cli');
export const CONFIG_PATH = path.join(CONFIG_DIR, 'config.json');
