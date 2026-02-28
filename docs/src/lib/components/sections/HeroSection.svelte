<script lang="ts">
	import { Icon, Bolt, InformationCircle } from 'svelte-hero-icons';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Snippet } from '$lib/components/ui/snippet/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import {
		commandPresets,
		benchmarks,
		benchmarkRows,
		type CommandPreset,
		type BenchScenario
	} from '$lib/data/page-data.js';

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

<section class="hero" id="top">
	<div class="hero-grid container">
		<div class="hero-copy">
			<Badge variant="outline" class="hero-badge">A Blazing-Fast Compiler for Svelte 5</Badge>
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
						<Snippet text={item.command} />
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

<style>
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

	@media (max-width: 980px) {
		.hero-grid {
			grid-template-columns: 1fr;
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
	}

	@media (max-width: 760px) {
		.hero {
			padding-top: 2rem;
		}
	}
</style>
