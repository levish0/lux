import process from 'node:process';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const luxCompile = vi.fn();

vi.mock('@lux/compiler', () => ({
	compile: luxCompile
}));

vi.mock('svelte/compiler', () => ({
	compile: vi.fn(() => {
		throw new Error('svelte compiler fallback should not run in this test');
	})
}));

vi.mock('../src/utils/log.js', () => ({
	log: {
		debug: Object.assign(vi.fn(), { enabled: false, once: vi.fn() }),
		info: Object.assign(vi.fn(), { enabled: false, once: vi.fn() }),
		warn: Object.assign(vi.fn(), { enabled: false, once: vi.fn() }),
		error: Object.assign(vi.fn(), { enabled: false, once: vi.fn() })
	}
}));

vi.mock('../src/utils/error.js', () => ({
	enhanceCompileError: vi.fn()
}));

const { createCompileSvelte, getLuxRuntimeModule } = await import('../src/utils/compile.js');

describe('createCompileSvelte with Lux', () => {
	beforeEach(() => {
		luxCompile.mockReset();
		process.env.LUX_SVELTE = '1';
		delete process.env.LUX_SVELTE_CLIENT;
	});

	afterEach(() => {
		delete process.env.LUX_SVELTE;
		delete process.env.LUX_SVELTE_CLIENT;
	});

	it('maps Lux compile output to Svelte CompileResult', async () => {
		luxCompile.mockReturnValue({
			js: 'export default {};',
			jsMap: JSON.stringify({
				version: 3,
				file: 'App.js',
				sources: ['/some/File.svelte'],
				names: [],
				mappings: ''
			}),
			css: 'h1{color:red}',
			cssMap: JSON.stringify({
				version: 3,
				file: 'App.css',
				sources: ['/some/File.svelte'],
				names: [],
				mappings: ''
			}),
			cssHash: 'abc123',
			cssScope: 'svelte-abc123',
			runtimeModules: [{ specifier: 'lux/runtime/server', code: 'export const ok = true;' }],
			errors: [],
			warnings: [{ code: 'test_warning', message: 'warn', start: 0, end: 0 }],
			metadataRunes: true,
			astJson: JSON.stringify({ type: 'Root', fragment: { nodeTypes: ['RegularElement'] } }),
			ts: false
		});

		const compileSvelte = createCompileSvelte();
		const output = await compileSvelte(
			{
				cssId: 'svelte-xxxxx',
				query: {},
				raw: false,
				ssr: true,
				timestamp: Date.now(),
				id: 'id',
				filename: '/some/File.svelte',
				normalizedFilename: 'some/File.svelte'
			},
			'<h1>ok</h1>',
			{
				compilerOptions: {
					runes: true,
					customElement: true,
					preserveWhitespace: true,
					css: 'injected'
				},
				root: process.cwd()
			}
		);

		expect(luxCompile).toHaveBeenCalledWith('<h1>ok</h1>', {
			ts: false,
			generate: 'server',
			filename: '/some/File.svelte',
			runes: true,
			customElement: true,
			preserveWhitespace: true,
			css: 'injected'
		});
		expect(output.compiled.js.map).toMatchObject({
			file: 'File.svelte',
			sources: ['File.svelte']
		});
		expect(output.compiled.css).toMatchObject({
			code: 'h1{color:red}',
			hasGlobal: false,
			map: {
				file: 'File.svelte',
				sources: ['File.svelte']
			}
		});
		expect(output.compiled.metadata).toEqual({ runes: true });
		expect(output.compiled.ast).toEqual({
			type: 'Root',
			fragment: { nodeTypes: ['RegularElement'] }
		});
		expect(output.compiled.warnings[0].code).toBe('test_warning');
		expect(getLuxRuntimeModule('lux/runtime/server')).toBe('export const ok = true;');
	});
});
