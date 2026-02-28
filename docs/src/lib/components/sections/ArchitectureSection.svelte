<script lang="ts">
	import { Icon, Folder, BookOpen, CpuChip, ChevronDoubleRight } from 'svelte-hero-icons';
	import * as Card from '$lib/components/ui/card/index.js';
</script>

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
						Svelte source under `reference/` stays the behavior baseline. Lux implements phase logic
						against that baseline rather than diverging API contracts.
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
					<p>Generates JS/CSS output from analyzed state while preserving compatibility intent.</p>
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</section>

<style>
	.section {
		padding: 4.5rem 0;
	}

	/* Stronger contrast for alt sections */
	.section-alt {
		background: color-mix(in srgb, var(--background) 60%, var(--muted));
		border-top: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
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

	@media (max-width: 980px) {
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
		.section {
			padding: 3rem 0;
		}
	}
</style>
