<script lang="ts">
  let items = [1, 2, 3];
</script>

{#each items as item: number}
  <p>{item}</p>
{/each}
