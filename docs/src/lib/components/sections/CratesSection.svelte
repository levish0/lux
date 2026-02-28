<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { crateRows } from '$lib/data/page-data.js';
</script>

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

	:global(.crates-card) :global(td code) {
		font-size: 0.75rem;
		color: var(--primary);
	}

	@media (max-width: 760px) {
		.section {
			padding: 3rem 0;
		}
	}
</style>
