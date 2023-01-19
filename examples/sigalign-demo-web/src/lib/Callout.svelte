<script lang="ts">
  import { slide } from 'svelte/transition';

  let toggled = false;
</script>

<main>
  <div class="callout">
    <button class="callout_toggle" on:click={() => { toggled = !toggled }}>
      💡
      <slot name="title">
        Title
      </slot>
      {#if toggled}
      ▲
      {:else}
      ▼
      {/if}
    </button>
    {#if toggled}
      <p class="callout_inner" transition:slide>
        <slot name="contents">
          Contents
        </slot>
      </p>  
    {/if}
  </div>
</main>

<style>
  .callout {
    margin: 0.5rem 2rem;
    background: #f2f1f1;
    padding: 0.5rem 1rem 0.5rem 1rem;
    border-radius: 0.5rem;
  }
  @media screen and (max-width: 720px) {
    .callout {
      margin: 0.5rem 0rem;
    }
  }
  button.callout_toggle {
    color: #213547;
    font-family: inherit;
    font-size: medium;
    border: 0rem;
    background: none;
  }
  .callout_inner {
    margin: 0.3rem 0rem 0rem 1rem;
  }
</style>