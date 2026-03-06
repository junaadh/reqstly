import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		// Keep dev dependency optimization deterministic to avoid stale hashed module URLs
		// during container startup/reload behind the reverse proxy.
		holdUntilCrawlEnd: true,
		exclude: ['@supabase/supabase-js']
	},
	server: {
		watch: {
			usePolling: process.env.VITE_USE_POLLING === 'true',
			interval: Number(process.env.VITE_POLLING_INTERVAL ?? '300')
		}
	}
});
