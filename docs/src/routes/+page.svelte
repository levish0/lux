<script lang="ts">
	import { mode, toggleMode } from 'mode-watcher';
	import {
		Icon,
		ArrowTopRightOnSquare,
		Bolt,
		BookOpen,
		ChevronDoubleRight,
		CodeBracketSquare,
		CommandLine,
		CpuChip,
		Folder,
		InformationCircle,
		Moon,
		Sun
	} from 'svelte-hero-icons';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { PMCommand } from '$lib/components/ui/pm-command/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Snippet } from '$lib/components/ui/snippet/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';

	type CommandPreset = 'quickstart' | 'parser' | 'analyzer';
	type BenchScenario = 'mean' | 'median' | 'ci';

	const commandPresets: Array<{
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

	const benchmarks: Record<
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

	const benchmarkRows = [
		{ name: 'Mean', svelte: 2.4075, lux: 0.0155, speedup: 155.67 },
		{ name: 'Median', svelte: 2.2354, lux: 0.014, speedup: 159.83 },
		{ name: '95% CI floor', svelte: 2.1302, lux: 0.0171, speedup: 124.7 }
	] as const;

	const crateRows = [
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

	const workflowBlocks = [
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

	let commandPreset = $state<CommandPreset>('quickstart');
	let benchScenario = $state<BenchScenario>('mean');

	const activeInstall = $derived(
		commandPresets.find((item) => item.value === commandPreset) ?? commandPresets[0]
	);
	const activeBenchmark = $derived(benchmarks[benchScenario]);

	function barWidth(value: number, max: number): string {
		if (max <= 0) return '0%';
		return `${(value / max) * 100}%`;
	}

	function formatMs(value: number): string {
		return `${value.toFixed(4)}ms`;
	}
</script>

<svelte:head>
	<title>lux docs | Svelte compiler in Rust</title>
	<meta
		name="description"
		content="Lux is a Rust implementation of the Svelte compiler pipeline (parse, analyze, transform), targeting compatibility with official Svelte output."
	/>
</svelte:head>

<div class="scanline" aria-hidden="true"></div>

<div class="page-shell">
	<header class="site-header">
		<div class="header-inner container">
			<a href="#top" class="brand">
				<span class="brand-mark">$</span>
				<span>lux compiler</span>
			</a>
			<nav class="header-nav">
				<a href="#use-case">Use Case</a>
				<a href="#architecture">Architecture</a>
				<a href="#install">Workflow</a>
				<a href="#crates">Crates</a>
			</nav>
			<div class="header-actions">
				<Button
					variant="outline"
					size="icon-sm"
					class="theme-button"
					onclick={toggleMode}
					aria-label="Toggle theme"
				>
					{#if mode.current === 'dark'}
						<Icon src={Sun} solid class="size-4" />
					{:else}
						<Icon src={Moon} solid class="size-4" />
					{/if}
				</Button>
				<Button
					variant="ghost"
					size="sm"
					href="https://github.com/levish0/lux"
					target="_blank"
					rel="noreferrer"
					class="repo-link"
				>
					<Icon src={CodeBracketSquare} solid class="size-4" />
					<span>GitHub</span>
				</Button>
			</div>
		</div>
	</header>

	<main>
		<section class="hero" id="top">
			<div class="hero-grid container">
				<div class="hero-copy">
					<Badge variant="outline" class="hero-badge">Svelte 5 compiler in Rust</Badge>
					<h1>
						Drop-in behavior target.<br />
						<span>Native parser speed.</span>
					</h1>
					<p class="hero-desc">
						Lux keeps the Svelte compiler pipeline explicit: parse, analyze, and transform run as
						separate Rust crates so we can optimize each stage without losing compatibility focus.
					</p>

					<Tabs.Root bind:value={commandPreset} class="install-tabs-shell">
						<Tabs.List class="install-tab-list">
							{#each commandPresets as item (item.value)}
								<Tabs.Trigger value={item.value}>{item.label}</Tabs.Trigger>
							{/each}
						</Tabs.List>

						{#each commandPresets as item (item.value)}
							<Tabs.Content value={item.value}>
								<div class="install-command" data-copy={item.command}>
									<span class="prompt">{item.prompt}</span>
									<code>{item.command}</code>
									<Button
										variant="ghost"
										size="icon-sm"
										class="copy-button"
										onclick={() => copyToClipboard(item.command, `install-${item.value}`)}
										aria-label={`Copy ${item.label} command`}
									>
										{#if copiedKey === `install-${item.value}`}
											<Icon src={Check} solid class="size-4" />
										{:else}
											<Icon src={ClipboardDocument} solid class="size-4" />
										{/if}
									</Button>
								</div>
							</Tabs.Content>
						{/each}
					</Tabs.Root>

					<p class="hero-note">{activeInstall.note}</p>
				</div>

				<Card.Root class="bench-card" id="benchmark">
					<Card.Header class="bench-header">
						<div class="bench-header-row">
							<div>
								<Card.Title class="bench-title">Parser benchmark snapshot</Card.Title>
								<Card.Description class="bench-subtitle"
									>Source: `benchmarks/criterion/lux-parser/parser`.</Card.Description
								>
							</div>
							<Dialog.Root>
								<Dialog.Trigger class="method-trigger">
									<Icon src={InformationCircle} solid class="size-3.5" />
									<span>Method</span>
								</Dialog.Trigger>
								<Dialog.Content class="method-dialog">
									<Dialog.Header>
										<Dialog.Title>Benchmark Methodology</Dialog.Title>
										<Dialog.Description>
											Criterion result files committed in this repo were used directly.
										</Dialog.Description>
									</Dialog.Header>
									<div class="method-metrics">
										<div>
											<span>Input file</span>
											<strong>benchmarks/assets/benchmark.svelte</strong>
										</div>
										<div>
											<span>File size</span>
											<strong>2,234 bytes / 62 lines</strong>
										</div>
										<div>
											<span>CI speed range</span>
											<strong>124.7x - 192.4x</strong>
										</div>
									</div>
									<Table.Root class="method-table">
										<Table.Header>
											<Table.Row>
												<Table.Head>Metric</Table.Head>
												<Table.Head>Svelte</Table.Head>
												<Table.Head>Lux</Table.Head>
												<Table.Head>Speedup</Table.Head>
											</Table.Row>
										</Table.Header>
										<Table.Body>
											{#each benchmarkRows as row (row.name)}
												<Table.Row>
													<Table.Cell>{row.name}</Table.Cell>
													<Table.Cell>{formatMs(row.svelte)}</Table.Cell>
													<Table.Cell>{formatMs(row.lux)}</Table.Cell>
													<Table.Cell class="text-primary">{row.speedup.toFixed(1)}x</Table.Cell>
												</Table.Row>
											{/each}
										</Table.Body>
									</Table.Root>
								</Dialog.Content>
							</Dialog.Root>
						</div>

						<Tabs.Root bind:value={benchScenario} class="bench-tabs-shell">
							<Tabs.List class="bench-tab-list">
								<Tabs.Trigger value="mean">Mean</Tabs.Trigger>
								<Tabs.Trigger value="median">Median</Tabs.Trigger>
								<Tabs.Trigger value="ci">95% CI Floor</Tabs.Trigger>
							</Tabs.List>
						</Tabs.Root>
					</Card.Header>

					<Card.Content class="bench-body">
						{@const maxValue = Math.max(activeBenchmark.svelte, activeBenchmark.lux)}
						<div class="bench-row">
							<div class="bench-tool">svelte parser path</div>
							<div class="bench-track">
								<div
									class="bench-fill bench-fill-baseline"
									style={`width: ${barWidth(activeBenchmark.svelte, maxValue)}`}
								></div>
							</div>
							<div class="bench-time">{formatMs(activeBenchmark.svelte)}</div>
						</div>

						<div class="bench-row">
							<div class="bench-tool">lux-parser</div>
							<div class="bench-track">
								<div
									class="bench-fill bench-fill-fast"
									style={`width: ${barWidth(activeBenchmark.lux, maxValue)}`}
								></div>
							</div>
							<div class="bench-time bench-time-fast">{formatMs(activeBenchmark.lux)}</div>
						</div>

						<div class="bench-summary">
							<div class="speed-pill">
								<Icon src={Bolt} solid class="size-4" />
								<strong>{activeBenchmark.speedup.toFixed(1)}x faster</strong>
							</div>
							<p>{activeBenchmark.detail}</p>
						</div>

						<div class="bench-meta">
							<span>{activeBenchmark.label}</span>
							<span class="dot" aria-hidden="true">.</span>
							<span>{activeBenchmark.runs}</span>
						</div>
					</Card.Content>
				</Card.Root>
			</div>
		</section>

		<Separator class="section-separator" />

		<section class="section" id="use-case">
			<div class="container">
				<h2 class="section-title">Built for compiler iteration loops</h2>
				<p class="section-lead">
					Lux splits the compiler into focused crates so parser, analyzer, and transformer work can
					be benchmarked and evolved independently.
				</p>

				<div class="use-grid">
					<Card.Root class="use-card use-card-good">
						<Card.Header>
							<Card.Title>Good fit</Card.Title>
							<Card.Description
								>When tight feedback and micro-bench visibility matter.</Card.Description
							>
						</Card.Header>
						<Card.Content>
							<ul>
								<li>Compiler feature parity work against upstream Svelte behavior</li>
								<li>Profiling parser/analyzer hotspots in isolation</li>
								<li>CI checks for parser and analyzer regressions</li>
								<li>Agentic coding workflows that need deterministic CLI output</li>
							</ul>
						</Card.Content>
					</Card.Root>

					<Card.Root class="use-card use-card-neutral">
						<Card.Header>
							<Card.Title>Not trying to replace</Card.Title>
							<Card.Description>
								Official Svelte compiler as the behavior source of truth.
							</Card.Description>
						</Card.Header>
						<Card.Content>
							<ul>
								<li>Upstream compiler design decisions and runtime behavior contracts</li>
								<li>IDE language tooling and editor integrations</li>
								<li>Production apps expecting stable upstream semantics</li>
								<li>Compatibility baselines documented in Svelte reference source</li>
							</ul>
						</Card.Content>
					</Card.Root>
				</div>
			</div>
		</section>

		<section class="section section-alt" id="architecture">
			<div class="container">
				<h2 class="section-title">Pipeline architecture</h2>
				<div class="approach-grid">
					<Card.Root class="approach-card">
						<Card.Header>
							<Card.Title>Reference-first</Card.Title>
						</Card.Header>
						<Card.Content>
							<p>
								Svelte source under `reference/` stays the behavior baseline. Lux implements phase
								logic against that baseline rather than diverging API contracts.
							</p>
						</Card.Content>
					</Card.Root>
					<div class="approach-vs">vs</div>
					<Card.Root class="approach-card approach-card-accent">
						<Card.Header>
							<Card.Title>Rust phase isolation</Card.Title>
						</Card.Header>
						<Card.Content>
							<p>
								Parser, analyzer, and transformer each expose focused crates. That makes per-phase
								benchmarking and regression checks straightforward in CI.
							</p>
						</Card.Content>
					</Card.Root>
				</div>

				<div class="stage-grid">
					<Card.Root class="stage-card">
						<Card.Header>
							<div class="stage-label"><Icon src={Folder} solid class="size-4" /> 01 Parse</div>
							<Card.Title>lux-parser</Card.Title>
						</Card.Header>
						<Card.Content>
							<p>Zero-copy parsing with winnow for templates/CSS and OXC for JS/TS expressions.</p>
						</Card.Content>
					</Card.Root>

					<div class="stage-arrow"><Icon src={ChevronDoubleRight} solid class="size-5" /></div>

					<Card.Root class="stage-card">
						<Card.Header>
							<div class="stage-label"><Icon src={BookOpen} solid class="size-4" /> 02 Analyze</div>
							<Card.Title>lux-analyzer</Card.Title>
						</Card.Header>
						<Card.Content>
							<p>Semantic passes build scopes, bindings, and diagnostics over the parsed AST.</p>
						</Card.Content>
					</Card.Root>

					<div class="stage-arrow"><Icon src={ChevronDoubleRight} solid class="size-5" /></div>

					<Card.Root class="stage-card">
						<Card.Header>
							<div class="stage-label">
								<Icon src={CpuChip} solid class="size-4" /> 03 Transform
							</div>
							<Card.Title>lux-transformer</Card.Title>
						</Card.Header>
						<Card.Content>
							<p>
								Generates JS/CSS output from analyzed state while preserving compatibility intent.
							</p>
						</Card.Content>
					</Card.Root>
				</div>
			</div>
		</section>

		<section class="section" id="install">
			<div class="container">
				<h2 class="section-title">Contributor workflow</h2>
				<div class="workflow-grid">
					{#each workflowBlocks as block (block.id)}
						<Card.Root class="workflow-card">
							<Card.Header>
								<Card.Title>{block.title}</Card.Title>
							</Card.Header>
							<Card.Content>
								<div class="workflow-command">
									<pre><code>{block.command}</code></pre>
									<Button
										variant="ghost"
										size="icon-sm"
										class="copy-button"
										onclick={() => copyToClipboard(block.command, `workflow-${block.id}`)}
										aria-label={`Copy ${block.title} command`}
									>
										{#if copiedKey === `workflow-${block.id}`}
											<Icon src={Check} solid class="size-4" />
										{:else}
											<Icon src={ClipboardDocument} solid class="size-4" />
										{/if}
									</Button>
								</div>
							</Card.Content>
						</Card.Root>
					{/each}
				</div>
			</div>
		</section>

		<section class="section section-alt" id="crates">
			<div class="container">
				<h2 class="section-title">Workspace crates</h2>
				<Card.Root class="crates-card">
					<Card.Content>
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Crate</Table.Head>
									<Table.Head>Role</Table.Head>
									<Table.Head>Phase</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each crateRows as row (row.name)}
									<Table.Row>
										<Table.Cell><code>{row.name}</code></Table.Cell>
										<Table.Cell>{row.desc}</Table.Cell>
										<Table.Cell>{row.phase}</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</Card.Content>
				</Card.Root>
			</div>
		</section>
	</main>

	<footer class="site-footer">
		<div class="footer-inner container">
			<div class="footer-left">
				<Icon src={CommandLine} solid class="size-4" />
				<span>Lux compiler docs</span>
			</div>
			<a href="https://github.com/levish0/lux" target="_blank" rel="noreferrer" class="footer-link">
				<span>Repository</span>
				<Icon src={ArrowTopRightOnSquare} solid class="size-3.5" />
			</a>
		</div>
	</footer>
</div>

<style>
	:global(html) {
		scroll-behavior: smooth;
	}

	:global(body) {
		font-feature-settings: 'ss03' 1;
	}

	.scanline {
		position: fixed;
		inset: 0;
		pointer-events: none;
		z-index: 100;
		opacity: 0.35;
		background: repeating-linear-gradient(
			0deg,
			transparent,
			transparent 2px,
			rgba(180, 180, 180, 0.025) 2px,
			rgba(180, 180, 180, 0.025) 4px
		);
	}

	:global(.dark) .scanline {
		opacity: 0.22;
	}

	.page-shell {
		position: relative;
		min-height: 100vh;
	}

	.container {
		width: min(1120px, calc(100% - 3rem));
		margin: 0 auto;
	}

	.site-header {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 80;
		border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
		backdrop-filter: blur(10px);
		background: color-mix(in srgb, var(--background) 78%, transparent);
	}

	.header-inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 62px;
		gap: 1rem;
	}

	.brand {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.875rem;
		font-weight: 600;
		letter-spacing: -0.02em;
	}

	.brand-mark {
		color: var(--primary);
	}

	.header-nav {
		display: flex;
		gap: 1rem;
	}

	.header-nav a {
		font-size: 0.8rem;
		color: var(--muted-foreground);
		transition: color 0.15s ease;
	}

	.header-nav a:hover {
		color: var(--foreground);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}

	:global(.theme-button) {
		border-radius: 0.5rem;
	}

	:global(.repo-link) {
		color: var(--muted-foreground);
	}

	main {
		padding-top: 84px;
	}

	.hero {
		padding: 3rem 0 4rem;
	}

	.hero-grid {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		gap: 2rem;
		align-items: start;
	}

	.hero-copy {
		padding-top: 1.25rem;
	}

	:global(.hero-badge) {
		margin-bottom: 1.1rem;
		border-color: color-mix(in srgb, var(--primary) 35%, var(--border));
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		font-size: 0.65rem;
	}

	h1 {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-weight: 700;
		font-size: clamp(1.95rem, 4.2vw, 2.75rem);
		line-height: 1.1;
		letter-spacing: -0.04em;
		margin-bottom: 1rem;
	}

	h1 span {
		color: var(--primary);
	}

	.hero-desc {
		color: var(--muted-foreground);
		line-height: 1.75;
		max-width: 58ch;
		margin-bottom: 1.35rem;
	}

	:global(.install-tabs-shell) {
		max-width: 100%;
		gap: 0.75rem;
	}

	:global(.install-tab-list) {
		width: fit-content;
	}

	.install-command {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		padding: 0.95rem 0.85rem 0.95rem 1rem;
		border-radius: 0.65rem;
		border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
		background: color-mix(in srgb, var(--card) 85%, transparent);
		overflow: hidden;
	}

	.install-command .prompt {
		color: var(--primary);
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.86rem;
		font-weight: 600;
		flex-shrink: 0;
	}

	.install-command code {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.8rem;
		line-height: 1.5;
		white-space: nowrap;
		overflow-x: auto;
		flex: 1;
	}

	:global(.copy-button) {
		flex-shrink: 0;
	}

	.hero-note {
		margin-top: 0.8rem;
		font-size: 0.82rem;
		color: var(--muted-foreground);
	}

	:global(.bench-card) {
		padding-bottom: 0.25rem;
		background: color-mix(in srgb, var(--card) 88%, transparent);
	}

	:global(.bench-header) {
		gap: 1rem;
	}

	.bench-header-row {
		display: flex;
		justify-content: space-between;
		gap: 1rem;
		align-items: flex-start;
	}

	:global(.bench-title) {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.98rem;
	}

	:global(.bench-subtitle) {
		margin-top: 0.4rem;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.72rem;
	}

	:global(.method-trigger) {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		border: 1px solid var(--border);
		border-radius: 0.5rem;
		padding: 0.35rem 0.5rem;
		font-size: 0.72rem;
		color: var(--muted-foreground);
		background: transparent;
	}

	:global(.method-trigger:hover) {
		color: var(--foreground);
		border-color: color-mix(in srgb, var(--foreground) 18%, var(--border));
	}

	:global(.method-dialog) {
		max-width: 720px;
	}

	.method-metrics {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 0.75rem;
		margin-top: 0.75rem;
		margin-bottom: 1rem;
	}

	.method-metrics div {
		padding: 0.75rem;
		border-radius: 0.65rem;
		border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
		background: color-mix(in srgb, var(--muted) 40%, transparent);
	}

	.method-metrics span {
		display: block;
		font-size: 0.72rem;
		color: var(--muted-foreground);
		margin-bottom: 0.3rem;
	}

	.method-metrics strong {
		font-size: 0.8rem;
		line-height: 1.35;
	}

	:global(.bench-tabs-shell) {
		gap: 0.6rem;
	}

	:global(.bench-tab-list) {
		width: fit-content;
	}

	:global(.bench-body) {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.bench-row {
		display: grid;
		grid-template-columns: 150px 1fr 88px;
		align-items: center;
		gap: 0.75rem;
	}

	.bench-tool {
		font-size: 0.75rem;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		color: var(--muted-foreground);
		text-align: right;
	}

	.bench-track {
		height: 1.55rem;
		border-radius: 0.45rem;
		background: color-mix(in srgb, var(--background) 88%, var(--card));
		overflow: hidden;
		position: relative;
	}

	.bench-fill {
		height: 100%;
		transition: width 0.35s ease;
	}

	.bench-fill-baseline {
		background: linear-gradient(
			90deg,
			color-mix(in srgb, var(--muted-foreground) 70%, transparent),
			color-mix(in srgb, var(--muted-foreground) 45%, transparent)
		);
	}

	.bench-fill-fast {
		background: linear-gradient(
			90deg,
			color-mix(in srgb, var(--primary) 95%, transparent),
			color-mix(in srgb, var(--primary) 70%, #5858ff)
		);
		box-shadow: 0 0 14px color-mix(in srgb, var(--primary) 35%, transparent);
	}

	.bench-time {
		font-size: 0.81rem;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		text-align: right;
	}

	.bench-time-fast {
		color: var(--primary);
		font-weight: 600;
	}

	.bench-summary {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.75rem;
		margin-top: 0.2rem;
	}

	.speed-pill {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.4rem 0.6rem;
		border-radius: 999px;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.8rem;
		border: 1px solid color-mix(in srgb, var(--primary) 40%, var(--border));
		background: color-mix(in srgb, var(--primary) 12%, transparent);
	}

	.bench-summary p {
		font-size: 0.8rem;
		color: var(--muted-foreground);
	}

	.bench-meta {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.72rem;
		color: var(--muted-foreground);
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
	}

	.dot {
		opacity: 0.5;
	}

	:global(.section-separator) {
		width: min(1120px, calc(100% - 3rem));
		margin: 0 auto;
	}

	.section {
		padding: 4.5rem 0;
	}

	.section-alt {
		background: color-mix(in srgb, var(--background) 88%, var(--card));
	}

	.section-title {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: clamp(1.2rem, 2.4vw, 1.5rem);
		letter-spacing: -0.03em;
		margin-bottom: 1.4rem;
	}

	.section-title::before {
		content: '# ';
		opacity: 0.5;
	}

	.section-lead {
		max-width: 72ch;
		line-height: 1.7;
		color: var(--muted-foreground);
		margin-bottom: 1.5rem;
	}

	.use-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 1rem;
	}

	:global(.use-card) ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 0.55rem;
	}

	:global(.use-card) li {
		position: relative;
		padding-left: 1rem;
		font-size: 0.87rem;
		line-height: 1.5;
		color: var(--muted-foreground);
	}

	:global(.use-card) li::before {
		content: '.';
		position: absolute;
		left: 0;
		top: -0.05rem;
		font-size: 1.1rem;
		opacity: 0.6;
	}

	:global(.use-card-good) {
		border-color: color-mix(in srgb, var(--primary) 40%, var(--border));
		box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 15%, transparent);
	}

	:global(.use-card-neutral) {
		background: color-mix(in srgb, var(--muted) 33%, transparent);
	}

	.approach-grid {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 1rem;
		align-items: center;
		margin-bottom: 1.1rem;
	}

	:global(.approach-card) p {
		font-size: 0.88rem;
		line-height: 1.6;
		color: var(--muted-foreground);
	}

	:global(.approach-card-accent) {
		border-color: color-mix(in srgb, var(--primary) 40%, var(--border));
	}

	.approach-vs {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.74rem;
		opacity: 0.65;
		text-transform: uppercase;
	}

	.stage-grid {
		display: flex;
		align-items: stretch;
		gap: 0.9rem;
	}

	:global(.stage-card) {
		flex: 1;
	}

	.stage-label {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.72rem;
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--primary);
	}

	.stage-arrow {
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0.45;
	}

	:global(.stage-card) p {
		font-size: 0.86rem;
		line-height: 1.6;
		color: var(--muted-foreground);
	}

	.workflow-grid {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 1rem;
	}

	.workflow-command {
		display: flex;
		gap: 0.5rem;
		align-items: flex-start;
		padding: 0.75rem;
		border-radius: 0.6rem;
		border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
		background: color-mix(in srgb, var(--muted) 26%, transparent);
	}

	.workflow-command pre {
		margin: 0;
		flex: 1;
		overflow-x: auto;
	}

	.workflow-command code {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.75rem;
		line-height: 1.7;
		white-space: pre;
	}

	:global(.crates-card) :global(td code) {
		font-size: 0.75rem;
		color: var(--primary);
	}

	.site-footer {
		padding: 2rem 0;
		border-top: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
	}

	.footer-inner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
		font-size: 0.82rem;
		color: var(--muted-foreground);
	}

	.footer-left {
		display: inline-flex;
		align-items: center;
		gap: 0.45rem;
	}

	.footer-link {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
	}

	.footer-link:hover {
		color: var(--foreground);
	}

	@media (max-width: 1100px) {
		.workflow-grid {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 980px) {
		.hero-grid {
			grid-template-columns: 1fr;
		}

		.use-grid {
			grid-template-columns: 1fr;
		}

		.approach-grid {
			grid-template-columns: 1fr;
		}

		.approach-vs {
			text-align: center;
		}

		.stage-grid {
			flex-direction: column;
		}

		.stage-arrow {
			transform: rotate(90deg);
		}
	}

	@media (max-width: 760px) {
		.container {
			width: min(1120px, calc(100% - 1.5rem));
		}

		.header-nav {
			display: none;
		}

		:global(.repo-link) span {
			display: none;
		}

		.hero {
			padding-top: 2rem;
		}

		.install-command code {
			font-size: 0.73rem;
		}

		.bench-row {
			grid-template-columns: 92px 1fr 72px;
			gap: 0.45rem;
		}

		.bench-tool {
			text-align: left;
			font-size: 0.66rem;
		}

		.bench-time {
			font-size: 0.68rem;
		}

		.bench-summary {
			flex-direction: column;
			align-items: flex-start;
		}

		.method-metrics {
			grid-template-columns: 1fr;
		}

		.section {
			padding: 3rem 0;
		}

		.footer-inner {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
