<script lang="ts">
  import { onMount } from "svelte";

  let count = $state(0);
  let name = $state("world");
  let items = $state([1, 2, 3]);
  let show = $state(true);
  let promise = $state(fetch("/api"));

  function increment() {
    count += 1;
  }
</script>

<svelte:head>
  <title>Hello {name}!</title>
</svelte:head>

<svelte:window on:keydown={(e) => console.log(e)} />

<div class="container" class:active={count > 0} style:color="red">
  <h1>Hello {name}!</h1>

  <button on:click={increment}>
    clicks: {count}
  </button>

  <input bind:value={name} placeholder="enter name" />

  {#if show}
    <p>Visible!</p>
  {:else}
    <p>Hidden!</p>
  {/if}

  {#each items as item, i (item)}
    <span>{i}: {item}</span>
  {:else}
    <p>No items</p>
  {/each}

  {#await promise}
    <p>Loading...</p>
  {:then data}
    <p>Loaded: {data}</p>
  {:catch error}
    <p>Error: {error}</p>
  {/await}

  {#key count}
    <p>Count is {count}</p>
  {/key}

  {@html "<strong>raw html</strong>"}
  {@const doubled = count * 2}
  {@debug count, name}
</div>

<style>
  .container {
    padding: 1rem;
  }
  .active {
    background: lightblue;
  }
  h1 {
    color: purple;
  }
</style>
