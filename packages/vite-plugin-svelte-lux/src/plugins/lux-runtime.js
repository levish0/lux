import { getLuxRuntimeModule } from '../utils/compile.js';

const VIRTUAL_PREFIX = '\0lux-runtime:';

/**
 * @returns {import('vite').Plugin}
 */
export function luxRuntime() {
	return {
		name: 'vite-plugin-svelte:lux-runtime',
		resolveId(id) {
			if (process.env.LUX_SVELTE !== '1') {
				return null;
			}

			if (!getLuxRuntimeModule(id)) {
				return null;
			}

			return VIRTUAL_PREFIX + id;
		},
		load(id) {
			if (!id.startsWith(VIRTUAL_PREFIX)) {
				return null;
			}

			const specifier = id.slice(VIRTUAL_PREFIX.length);
			return getLuxRuntimeModule(specifier) ?? null;
		}
	};
}
