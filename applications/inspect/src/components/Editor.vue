<script setup lang="ts">
import {ref} from 'vue';
import init, {parse_doc, greet} from 'cdoc-wasm';
// import {JsonViewer} from 'vue-json-viewer'
init();

function do_parse() {
  ast.value = parse_doc(source.value)
}


const source = ref("")
const ast = ref({});
</script>

<template>
  <textarea v-model="source"></textarea>
  <button @click="do_parse">Parse</button>
  <json-viewer :value="ast"></json-viewer>

</template>

<style scoped>

</style>