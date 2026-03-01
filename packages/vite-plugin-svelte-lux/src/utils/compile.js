import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import * as svelte from 'svelte/compiler';
import { compile as luxCompile } from '@lux/compiler';
import { log } from './log.js';

import { mapToRelative } from './sourcemaps.js';
import { enhanceCompileError } from './error.js';

// TODO this is a patched version of https://github.com/sveltejs/vite-plugin-svelte/pull/796/files#diff-3bce0b33034aad4b35ca094893671f7e7ddf4d27254ae7b9b0f912027a001b15R10
// which is closer to the other regexes in at least not falling into commented script
// but ideally would be shared exactly with svelte and other tools that use it
const scriptLangRE =
	/<!--[^]*?-->|<script\s+(?:[^>]*|(?:[^=>'"/]+=(?:"[^"]*"|'[^']*'|[^>\s]+)\s+)*)lang=(["'])?([^"' >]+)\1[^>]*>/g;

/** @type {Map<string, string>} */
const luxRuntimeModules = new Map();

/**
 * @param {string} specifier
 */
export function getLuxRuntimeModule(specifier) {
	return luxRuntimeModules.get(specifier);
}

/**
 * @returns {import('../types/compile.d.ts').CompileSvelte}
 */
export function createCompileSvelte() {
	/** @type {import('../types/vite-plugin-svelte-stats.d.ts').StatCollection | undefined} */
	let stats;
	/** @type {import('../types/compile.d.ts').CompileSvelte} */
	return async function compileSvelte(svelteRequest, code, options, sourcemap) {
		const { filename, normalizedFilename, cssId, ssr, raw } = svelteRequest;
		const { emitCss = true } = options;
		/** @type {import('svelte/compiler').Warning[]} */
		const warnings = [];

		if (options.stats) {
			if (options.isBuild) {
				if (!stats) {
					// build is either completely ssr or csr, create stats collector on first compile
					// it is then finished in the buildEnd hook.
					stats = options.stats.startCollection(`${ssr ? 'ssr' : 'dom'} compile`, {
						logInProgress: () => false
					});
				}
			} else {
				// dev time ssr, it's a ssr request and there are no stats, assume new page load and start collecting
				if (ssr && !stats) {
					stats = options.stats.startCollection('ssr compile');
				}
				// stats are being collected but this isn't an ssr request, assume page loaded and stop collecting
				if (!ssr && stats) {
					stats.finish();
					stats = undefined;
				}
				// TODO find a way to trace dom compile during dev
				// problem: we need to call finish at some point but have no way to tell if page load finished
				// also they for hmr updates too
			}
		}

		/** @type {import('svelte/compiler').CompileOptions} */
		const compileOptions = {
			...options.compilerOptions,
			filename,
			generate: ssr ? 'server' : 'client'
		};

		let finalCode = code;
		if (compileOptions.hmr && options.emitCss) {
			const closeStylePos = code.lastIndexOf('</style>');
			if (closeStylePos > -1) {
				// inject rule that forces compile to attach scope class to every node in the template
				// this reduces the amount of js hot updates when editing css in .svelte files
				finalCode = finalCode.slice(0, closeStylePos) + ' *{}' + finalCode.slice(closeStylePos);
			}
		}

		const dynamicCompileOptions = await options?.dynamicCompileOptions?.({
			filename,
			code: finalCode,
			compileOptions
		});
		if (dynamicCompileOptions && log.debug.enabled) {
			log.debug(
				`dynamic compile options for  ${filename}: ${JSON.stringify(dynamicCompileOptions)}`,
				undefined,
				'compile'
			);
		}
		const finalCompileOptions = dynamicCompileOptions
			? {
					...compileOptions,
					...dynamicCompileOptions
				}
			: compileOptions;
		if (sourcemap) {
			finalCompileOptions.sourcemap = sourcemap;
		}
		const endStat = stats?.start(filename);
		/** @type {import('svelte/compiler').CompileResult} */
		let compiled;
		try {
			if (shouldUseLuxCompiler(ssr)) {
				const luxOutput = compileWithLux(finalCode, filename);
				writeLuxArtifactsIfEnabled(
					luxOutput.result,
					filename,
					normalizedFilename,
					ssr,
					options
				);
				compiled = luxOutput.compiled;
			} else {
				compiled = svelte.compile(finalCode, { ...finalCompileOptions, filename });
			}

			// patch output with partial accept until svelte does it
			// TODO remove later
			if (
				options.server?.config.experimental.hmrPartialAccept &&
				compiled.js.code.includes('import.meta.hot.accept(')
			) {
				compiled.js.code = compiled.js.code.replaceAll(
					'import.meta.hot.accept(',
					'import.meta.hot.acceptExports(["default"],'
				);
			}
		} catch (e) {
			enhanceCompileError(e, code, options.preprocess);
			throw e;
		}

		if (endStat) {
			endStat();
		}
		mapToRelative(compiled.js?.map, filename);
		mapToRelative(compiled.css?.map, filename);
		if (warnings.length) {
			if (!compiled.warnings) {
				compiled.warnings = [];
			}
			compiled.warnings.push(...warnings);
		}
		if (!raw) {
			// wire css import and code for hmr
			const hasCss = compiled.css?.code?.trim()?.length ?? 0 > 0;
			// compiler might not emit css with mode none or it may be empty
			if (emitCss && hasCss) {
				// TODO properly update sourcemap?
				compiled.js.code += `\nimport ${JSON.stringify(cssId)};\n`;
			}
		}

		let lang = 'js';
		for (const match of code.matchAll(scriptLangRE)) {
			if (match[2]) {
				lang = match[2];
				break;
			}
		}

		return {
			filename,
			normalizedFilename,
			cssId,
			lang,
			compiled,
			ssr
		};
	};
}

/**
 * @param {boolean} ssr
 */
function shouldUseLuxCompiler(ssr) {
	return process.env.LUX_SVELTE === '1' && ssr;
}

/**
 * @param {string} code
 * @param {string} filename
 * @returns {{ compiled: import('svelte/compiler').CompileResult, result: any }}
 */
function compileWithLux(code, filename) {
	const result = luxCompile(code);
	for (const runtimeModule of result.runtimeModules ?? []) {
		luxRuntimeModules.set(runtimeModule.specifier, runtimeModule.code);
	}
	if (result.errors.length > 0) {
		throw buildLuxCompileError(result.errors[0], code, filename);
	}

	/** @type {import('svelte/compiler').Warning[]} */
	const warnings = result.warnings.map((warning) => {
		const start = offsetToLocation(code, warning.start);
		const end = offsetToLocation(code, warning.end);
		return {
			code: warning.code ?? 'lux_warning',
			message: warning.message,
			filename,
			start,
			end
		};
	});

	const compiled = {
		js: {
			code: result.js,
			map: null
		},
		css: result.css
			? {
					code: result.css,
					map: null
				}
			: null,
		warnings,
		metadata: {},
		ast: null
	};
	return { compiled, result };
}

/**
 * @param {{ code?: string | null, message: string, start: number, end: number }} diagnostic
 * @param {string} code
 * @param {string} filename
 */
function buildLuxCompileError(diagnostic, code, filename) {
	const error = new Error(diagnostic.message);
	error.name = 'LuxCompileError';
	error.filename = filename;
	error.code = diagnostic.code ?? 'lux_compile_error';
	error.start = offsetToLocation(code, diagnostic.start);
	error.end = offsetToLocation(code, diagnostic.end);
	return error;
}

/**
 * @param {string} code
 * @param {number} offset
 */
function offsetToLocation(code, offset) {
	const safe = Math.max(0, Math.min(offset, code.length));
	let line = 1;
	let lineStart = 0;

	for (let i = 0; i < safe; i++) {
		if (code.charCodeAt(i) === 10) {
			line += 1;
			lineStart = i + 1;
		}
	}

	return {
		line,
		column: safe - lineStart,
		character: safe
	};
}

/**
 * @param {any} result
 * @param {string} filename
 * @param {string} normalizedFilename
 * @param {boolean} ssr
 * @param {import('../types/options.d.ts').ResolvedOptions} options
 */
function writeLuxArtifactsIfEnabled(result, filename, normalizedFilename, ssr, options) {
	const outputDir = process.env.LUX_ARTIFACTS_DIR;
	if (!outputDir) {
		return;
	}

	let rel = normalizedFilename || filename;
	if (isAbsolute(rel)) {
		rel = relative(options.root, rel);
	}
	rel = sanitizeArtifactRelativePath(rel);

	const base = resolve(outputDir, `${rel}.${ssr ? 'server' : 'client'}`);
	mkdirSync(dirname(base), { recursive: true });
	writeFileSync(`${base}.js`, result.js ?? '', 'utf8');
	if (typeof result.css === 'string') {
		writeFileSync(`${base}.css`, result.css, 'utf8');
	}
	writeFileSync(
		`${base}.meta.json`,
		JSON.stringify(
			{
				filename,
				normalizedFilename,
				ts: result.ts ?? false,
				errors: result.errors ?? [],
				warnings: result.warnings ?? [],
				runtimeModules: (result.runtimeModules ?? []).map((module) => module.specifier)
			},
			null,
			2
		),
		'utf8'
	);
}

/**
 * Ensure generated artifact paths always stay under `.lux_artifacts`
 * even when Vite provides ids outside the project root.
 * @param {string} rel
 */
function sanitizeArtifactRelativePath(rel) {
	const segments = String(rel).replace(/\\/g, '/').split('/');
	const safeSegments = [];
	let escapedRoot = false;

	for (const segment of segments) {
		if (!segment || segment === '.') continue;
		if (segment === '..') {
			escapedRoot = true;
			continue;
		}
		safeSegments.push(segment.replace(/[:*?"<>|]/g, '_'));
	}

	if (safeSegments.length === 0) {
		safeSegments.push('unknown');
	}

	if (escapedRoot) {
		safeSegments.unshift('_external');
	}

	return safeSegments.join('/');
}
