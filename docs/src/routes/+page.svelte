<script lang="ts">
	import { mode, toggleMode } from 'mode-watcher';
	import { Icon, CommandLine, Moon, Sun, ArrowTopRightOnSquare } from 'svelte-hero-icons';
	import GithubIcon from '@lucide/svelte/icons/github';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import HeroSection from '$lib/components/sections/HeroSection.svelte';
	import UseCaseSection from '$lib/components/sections/UseCaseSection.svelte';
	import ArchitectureSection from '$lib/components/sections/ArchitectureSection.svelte';
	import WorkflowSection from '$lib/components/sections/WorkflowSection.svelte';
	import CratesSection from '$lib/components/sections/CratesSection.svelte';
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
					<GithubIcon class="size-4" />
					<span>GitHub</span>
				</Button>
			</div>
		</div>
	</header>

	<main>
		<HeroSection />
		<Separator class="section-separator" />
		<UseCaseSection />
		<ArchitectureSection />
		<WorkflowSection />
		<CratesSection />
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

	:global(.section-separator) {
		width: min(1120px, calc(100% - 3rem));
		margin: 0 auto;
	}

	/* Shared section title style — consumed by child sections via :global */
	:global(.section-title) {
		font-family: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: clamp(1.2rem, 2.4vw, 1.5rem);
		letter-spacing: -0.03em;
		margin-bottom: 1.4rem;
	}

	:global(.section-title)::before {
		content: '# ';
		opacity: 0.5;
	}

	:global(.section-lead) {
		max-width: 72ch;
		line-height: 1.7;
		color: var(--muted-foreground);
		margin-bottom: 1.5rem;
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

	@media (max-width: 760px) {
		.header-nav {
			display: none;
		}

		:global(.repo-link) span {
			display: none;
		}

		.footer-inner {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
