<template>
  <div>
    <img alt="Vue logo" src="./assets/logo.png">
    <Reference/>
    <Aligner/>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Reference from './components/Reference.vue';
import Aligner from './components/Aligner.vue';

export default defineComponent({
  name: 'App',
  components: {
    Reference,
    Aligner,
  }
});
</script>

<style lang="scss">
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
