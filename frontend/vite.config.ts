import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

const usePolling = process.env.VITE_USE_POLLING === 'true';
const pollingInterval = Number(process.env.VITE_POLLING_INTERVAL ?? '1000');

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		// Keep dev dependency optimization deterministic to avoid stale hashed module URLs
		// during container startup/reload behind the reverse proxy.
		holdUntilCrawlEnd: true
	},
	server: {
		watch: {
			usePolling,
			interval: pollingInterval,
			ignored: ['**/.git/**', '**/.svelte-kit/**', '**/node_modules/**']
		}
	}
});
