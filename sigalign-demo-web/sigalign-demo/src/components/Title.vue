<template>
  <div class="content is-medium mt-5 has-text-centered has-text-brown">
    <h1><span class="is-italic has-text-primary">Sigalign</span> Demo Web</h1>
    <p>
      <span class="has-text-primary has-text-weight-semibold">Si</span>milarity <span class="has-text-primary has-text-weight-semibold">g</span>uided sequence <span class="has-text-primary has-text-weight-semibold">align</span>ment algorithm</p>

    <block class="px-1">
      <button class="button is-rounded is-primary is-outlined" onclick="window.open('https://github.com/baku4/sigalign/');">
        <span class="icon">
          <i class="fab fa-github-alt"></i>
        </span>
        <span>GitHub</span>
      </button>
    </block>

    <block class="px-1">
      <button class="button is-rounded is-primary is-outlined" onclick="window.open('https://crates.io/crates/sigalign/');">
        <span class="icon">
          <i class="fab fa-rust"></i>
        </span>
        <span>Crate</span>
      </button>
    </block>

    <block class="px-1">
      <button class="button is-rounded is-primary is-outlined">
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