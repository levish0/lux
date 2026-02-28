<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
	import { Snippet } from '$lib/components/ui/snippet/index.js';
	import { workflowBlocks } from '$lib/data/page-data.js';
</script>

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
						<Snippet
							text={block.command.includes('\n')
								? block.command.split('\n')
								: block.command}
						/>
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	</div>
</section>

<style>
	.section {
		padding: 4.5rem 0;
	}

	.workflow-grid {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 1rem;
	}

	@media (max-width: 1100px) {
		.workflow-grid {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 760px) {
		.section {
			padding: 3rem 0;
		}
	}
</style>
