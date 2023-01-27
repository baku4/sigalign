<script lang="ts">
  import { blur } from 'svelte/transition';

  export let toggled: boolean = false;
  export let text: string = '';
</script>

<button class="callout_toggle" on:click={() => { toggled = !toggled }}>
  <div class="triangle">
    {#if toggled}
    ▼
    {:else}
    ▶
    {/if}
  </div>
  <slot name="text">
    {text}
  </slot>
</button>

<style>
  button.callout_toggle {
    font-family: inherit;
    padding: 0.4rem 0.7rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    background: none;

    transition: background-color 0.3s ease;
  }
  button.callout_toggle:hover {
    background-color: rgba(251,37,118,0.1)
  }
  button.callout_toggle:active {
    background-color: rgba(251,37,118,0.2)
  }
  div.triangle {
    display: inline-block;
    width: 1rem;
    font-size: 0.8rem;
  }
</style>