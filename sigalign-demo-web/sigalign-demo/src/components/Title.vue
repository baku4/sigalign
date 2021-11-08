<template>
  <div class="has-background-navy has-text-lightpink px-3 py-4">
    <block class="is-size-2">Sigalign Web Demo</block>
    <br>
    <block>https://github.com/baku4/sigalign</block>
    <br>
    <block>
      <button class="button is-lightpink is-normal is-rounded">Github</button>
      <button class="button is-lightpink is-normal is-rounded">Crate</button>
    </block>
    <br>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Title',
});
</script>