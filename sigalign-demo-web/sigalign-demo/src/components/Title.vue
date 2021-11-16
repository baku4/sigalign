<template>
  <div class="content is-medium mt-5 has-text-centered">
    <h1 class="has-text-centered"><span class="is-italic"><span class="has-text-primary">Sigalign</span> Demo Web</span></h1>
    <p>
      A demo of a web application using a <span class="is-italic">Sigalign</span> compiled to <span class="is-italic">WebAssembly</span>.<br>
      <span class="is-italic">Sigalign</span> is a sequence alignment algorithm and its implementation library.
    </p>

    <!-- Buttons -->
    <div class="has-text-centered">
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
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Title',
});
</script>