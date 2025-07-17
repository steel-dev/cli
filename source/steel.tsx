#!/usr/bin/env node
import Pastel from 'pastel';

const app = new Pastel({
	importMeta: import.meta,
	version: '0.0.1-alpha.6',
});

await app.run();
