<template>
  <div>
    <div class="columns">
      <div class="column is-size-5">
        Query:
      </div>
    </div>
    <button class="button is-navy is-normal">
     Semi-global
    </button>
    <button class="button is-navy is-normal">
      Local
    </button>
    <div class="columns">
      <div class="column">
        <button class="button is-navy is-normal">
          Semi-global
        </button>
        <button class="button is-navy is-normal">
          Local
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Alignment',
});
</script>