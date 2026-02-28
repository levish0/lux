<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
</script>

<section class="section" id="use-case">
	<div class="container">
		<h2 class="section-title">Built for compiler iteration loops</h2>
		<p class="section-lead">
			Lux splits the compiler into focused crates so parser, analyzer, and transformer work can be
			benchmarked and evolved independently.
		</p>

		<div class="use-grid">
			<Card.Root class="use-card use-card-good">
				<Card.Header>
					<Card.Title>Good fit</Card.Title>
					<Card.Description>When tight feedback and micro-bench visibility matter.</Card.Description>
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
					<Card.Description>Official Svelte compiler as the behavior source of truth.</Card.Description>
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

<style>
	.section {
		padding: 4.5rem 0;
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

	@media (max-width: 980px) {
		.use-grid {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 760px) {
		.section {
			padding: 3rem 0;
		}
	}
</style>
