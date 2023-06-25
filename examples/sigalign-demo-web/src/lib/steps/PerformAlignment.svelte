<script lang="ts">
  import TextArea from "../components/TextArea.svelte";
  import {
    type Reference,
    type Aligner,
    type AlignmentResult,
    get_sample_query,
  } from "../../wasm/sigalign_demo_wasm";

  export let reference: Reference;
  export let aligner: Aligner;
  export let alignmentResult: AlignmentResult;

  let query: string = "";
  let errorMsg: string = null;

  function checkPrepared(): boolean {
    if (reference === null) {
      if (aligner === null) {
        errorMsg = "<b><i>Reference</i></b> and <b><i>Aligner</i></b> are not ready."
        return false;
      } else {
        errorMsg = "<b><i>Reference</i></b> is not ready."
        return false;
      }
    } else {
      if (aligner === null) {
        errorMsg = "<b><i>Aligner</i></b> is not ready."
        return false;
      } 
    }
    return true;
  }
  function getSampleQuery() {
    let prepared = checkPrepared();
    if (!prepared) {
      return;
    }
    try {
      query = get_sample_query(reference, aligner);
      errorMsg = null;
    } catch (err) {
      errorMsg = err;
      query = "";
    }
  }
  function startAlignment() {
    let prepared = checkPrepared();
    if (!prepared) {
      return;
    }
    let promise = aligner.alignment(query, reference);
    promise.then((v) => {
      alignmentResult = v;
      errorMsg = null;
      console.log("Alignment Done");
    }).catch((err) => {
      errorMsg = err;
      console.log(err);
    })
  }
  function resetResults() {
    query = "";
    alignmentResult = null;
    errorMsg = null;
  }
</script>

<h3 class="header">🛠️ Pass the query</h3>

<!-- Query Text Area -->
{#if alignmentResult === null}
  <TextArea
    bind:value={query}
    height_rem={4}
  />
{:else}
  <div style="margin-top: 1rem;margin-left: 1rem;">Alignment is done.</div>
{/if}

<!-- Error Log -->
{#if errorMsg !== null}
  <div class="button-wrapper">
    {@html errorMsg}
  </div>
{/if}

<!-- Buttons -->
{#if alignmentResult === null}
  <div class="button-wrapper">
    <button class="default primary" on:click={getSampleQuery}>Get sample query</button>
  </div>
  <div class="button-wrapper">
    <button class="default primary" on:click={startAlignment}>Start Alignment</button>
  </div>
{:else}
  <div class="button-wrapper">
    <button class="default primary" on:click={resetResults}>Reset Results</button>
  </div>
{/if}

<style>
  
</style>