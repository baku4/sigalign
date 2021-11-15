<template>
  <div class="px-3 py-4">
    <block class="is-size-1">Sigalign Demo Web</block>
    <p>
      Similarity guided sequence alignment algorithm
    </p>

    <block class="px-1">
      <button class="button is-rounded">
        <span class="icon">
          <i class="fab fa-github-alt"></i>
        </span>
        <span>GitHub</span>
      </button>
    </block>

    <block class="px-1">
      <button class="button is-rounded">
        <span class="icon">
          <i class="fab fa-rust"></i>
        </span>
        <span>Crate</span>
      </button>
    </block>

    <block class="px-1">
      <button class="button is-rounded">
        <span class="icon">
          <i class="fab fa-python"></i>
        </span>
        <span>Pip</span>
      </button>
    </block>

  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Title',
});
</script>