<script lang="ts">
	import './layout.css';
	import { ModeWatcher } from 'mode-watcher';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<ModeWatcher defaultMode="dark" />
<svelte:head>
	<link rel="icon" href={favicon} />
	<meta name="theme-color" content="#0c0c0c" media="(prefers-color-scheme: dark)" />
	<meta name="theme-color" content="#f7f4ee" media="(prefers-color-scheme: light)" />
</svelte:head>
{@render children()}
