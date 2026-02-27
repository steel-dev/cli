import {defineConfig} from 'vitest/config';

export default defineConfig({
	test: {
		globals: true,
		testTimeout: 60_000,
		pool: 'forks',
		poolOptions: {
			forks: {
				singleFork: true,
			},
		},
	},
});
