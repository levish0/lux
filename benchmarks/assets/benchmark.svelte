<script lang="ts">
  export let items: Array<{ id: number; label: string; done: boolean }> = [];
  let filter = "";
  let selected = new Set<number>();

  const visible = $derived(
    items.filter((item) => item.label.toLowerCase().includes(filter.toLowerCase())),
  );

  function toggle(id: number) {
    if (selected.has(id)) {
      selected.delete(id);
    } else {
      selected.add(id);
    }
    selected = selected;
  }
</script>

<svelte:window on:keydown={(e) => e.key === "Escape" && (filter = "")} />

<section class="panel">
  <header>
    <h1>Bench Fixture</h1>
    <input bind:value={filter} placeholder="Filter items" />
  </header>

  {#if visible.length === 0}
    <p class="empty">No items</p>
  {:else}
    <ul>
      {#each visible as item, index (item.id)}
        <li class:done={item.done} on:click={() => toggle(item.id)}>
          <span>{index + 1}</span>
          <strong>{item.label}</strong>
          {#if selected.has(item.id)}
            <em>selected</em>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.25rem 0;
  }

  li.done strong {
    text-decoration: line-through;
  }

  .empty {
    opacity: 0.7;
  }
</style>
