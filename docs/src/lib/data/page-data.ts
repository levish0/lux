export type CommandPreset = 'quickstart' | 'parser' | 'analyzer';
export type BenchScenario = 'mean' | 'median' | 'ci';

export const commandPresets: Array<{
	value: CommandPreset;
	label: string;
	command: string;
	note: string;
}> = [
	{
		value: 'quickstart',
		label: 'Quick Start',
		command: 'cargo check --workspace',
		note: 'Compile every crate in the workspace.'
	},
	{
		value: 'parser',
		label: 'Parser Bench',
		command: 'cargo bench -p lux-parser --bench svelte_compare_parser',
		note: 'Compares Lux parse speed against the Svelte Node path.'
	},
	{
		value: 'analyzer',
		label: 'Analyzer Bench',
		command: 'cargo bench -p lux-analyzer --bench svelte_compare_analyzer',
		note: 'Runs analyzer-phase benchmark against Svelte analysis.'
	}
];

export const benchmarks: Record<
	BenchScenario,
	{
		label: string;
		runs: string;
		detail: string;
		svelte: number;
		lux: number;
		speedup: number;
	}
> = {
	mean: {
		label: 'Mean',
		runs: 'Criterion new/estimates.json',
		detail: 'Point estimate based on full sample distribution.',
		svelte: 2.4075,
		lux: 0.0155,
		speedup: 155.67
	},
	median: {
		label: 'Median',
		runs: 'Criterion new/estimates.json',
		detail: 'Median estimate to dampen outlier influence.',
		svelte: 2.2354,
		lux: 0.014,
		speedup: 159.83
	},
	ci: {
		label: '95% CI Floor',
		runs: 'Derived from confidence interval bounds',
		detail: 'Conservative floor using Svelte lower-bound / Lux upper-bound.',
		svelte: 2.1302,
		lux: 0.0171,
		speedup: 124.7
	}
};

export const benchmarkRows = [
	{ name: 'Mean', svelte: 2.4075, lux: 0.0155, speedup: 155.67 },
	{ name: 'Median', svelte: 2.2354, lux: 0.014, speedup: 159.83 },
	{ name: '95% CI floor', svelte: 2.1302, lux: 0.0171, speedup: 124.7 }
] as const;

export const crateRows = [
	{
		name: 'lux-ast',
		desc: 'AST type definitions + ESTree serialization bridge',
		phase: 'Shared Core'
	},
	{
		name: 'lux-parser',
		desc: 'winnow template/CSS parser + OXC JS/TS expression parsing',
		phase: 'Parse'
	},
	{
		name: 'lux-analyzer',
		desc: 'Scope/binding validation and semantic analysis',
		phase: 'Analyze'
	},
	{
		name: 'lux-transformer',
		desc: 'JS/CSS generation pipeline from analyzed AST',
		phase: 'Transform'
	},
	{
		name: 'lux-utils',
		desc: 'Keyword tables, fast maps, and shared helpers',
		phase: 'Shared Core'
	},
	{
		name: 'lux-metadata',
		desc: 'Shared Svelte metadata tables',
		phase: 'Shared Core'
	}
] as const;

export const workflowBlocks = [
	{
		id: 'clone',
		title: 'Clone and verify',
		command: 'git clone https://github.com/levish0/lux.git\ncd lux\ncargo check --workspace'
	},
	{
		id: 'parser-bench',
		title: 'Parser benchmark',
		command: 'cargo bench -p lux-parser --bench svelte_compare_parser'
	},
	{
		id: 'custom-input',
		title: 'Benchmark custom input',
		command:
			'$env:LUX_BENCH_INPUT="benchmarks/assets/benchmark.svelte"\ncargo bench -p lux-parser --bench svelte_compare_parser'
	}
] as const;
