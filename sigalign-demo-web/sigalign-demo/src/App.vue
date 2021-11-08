<template>
  <div class="has-text-font-color">
    <div class="container is-max-desktop has-text-navy">
      <!-- Title -->
      <div class="columns">
        <div class="column">
          <Title/>
        </div>
      </div>
      <!-- Preparation -->
      <div class="columns">
        <div class="column is-size-3">
          1. Preparation
        </div>
      </div>
      <Reference/>
      <Aligner/>
      <!-- Alignment -->
      <!-- <hr class="has-background-navy"> -->
      <div class="columns">
        <div class="column is-size-3">
          2. Alignment 
        </div>
      </div>
      <Alignment/>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Title from './components/Title.vue';
import Reference from './components/Reference.vue';
import Aligner from './components/Aligner.vue';
import Alignment from './components/Alignment.vue';

export default defineComponent({
  name: 'App',
  components: {
    Title,
    Reference,
    Aligner,
    Alignment,
  },
  data() {
    return {
      count: 0
    }
  },

});
</script>

<style lang="scss">
@import "./assets/style.scss";
</style>
