<script lang="ts">
  import TextArea from "../components/TextArea.svelte";
  import {
    type Reference,
    type ReferenceStatus,
    get_sample_target_as_fasta_string,
  } from "../../wasm/sigalign_demo_wasm";

  type BuildFnType = (
    fasta: string, sasr?: number, lts?: number
  ) => Promise<Reference>;

  export let reference: Reference;
  export let buildRefFn: BuildFnType;

  let fasta: string = "";

  let lts: number = null;
  let sasr: number = null;
  
  let errorMsg: string = null;

  let referenceStatus: ReferenceStatus = null;

  function getRandomFasta() {
    fasta = get_sample_target_as_fasta_string();
  }
  function buildReference() {
    if (fasta === "") {
      errorMsg = "FASTA string is empty.";
      return;
    }
    let promise = buildRefFn(fasta, sasr,lts);
    promise.then((v) => {
      reference = v;
      errorMsg = null;
      referenceStatus = reference.get_status();
      console.log("Reference built");
    }).catch((err) => {
      errorMsg = err;
      console.log(err);
    })
  }
  function resetReference() {
    lts = null;
    sasr = null;
    reference.drop();
    referenceStatus = null;
    reference = null;
  }
</script>

<h3 class="header">🛠️ Reference Builder</h3>

<!-- Descriptions -->
{#if reference === null}
  <p>
    In this tour, we will use <a href="https://en.wikipedia.org/wiki/FASTA_format" target="_blank" rel="noreferrer">FASTA</a> formatted string to define multiple sequences to use as a target.
  </p>
  <TextArea
    bind:value={fasta}
    height_rem={8}
  />

{:else}
  <div style="margin-top: 1rem;margin-left: 1rem;">Your <span class="highlight">Reference</span> is ready.
    {#if referenceStatus !== null}
      <div class="status">
        <ul>
          <li><b>Num. targets:</b> {referenceStatus.num_targets}</li>
          <li><b>Estimated size:</b>
          {#if referenceStatus.est_byte_size > 1_000_000_000 }
            {(referenceStatus.est_byte_size / 1_000_000_000).toFixed(2)} GiB
          {:else if referenceStatus.est_byte_size > 1_000_000}
            {(referenceStatus.est_byte_size / 1_000_000).toFixed(2)} MiB
          {:else if referenceStatus.est_byte_size > 1_000 }
            {(referenceStatus.est_byte_size / 1_000).toFixed(2)} KiB
          {:else}
          {referenceStatus.est_byte_size} B
          {/if}
          </li>
        </ul>
      </div>
    {/if}
  </div>
{/if}

<!-- Controller -->
{#if errorMsg !== null}
  <div class="button-wrapper">
    {errorMsg}
  </div>
{/if}
{#if reference === null}
  <div class="button-wrapper">
    <button class="default primary" on:click={getRandomFasta}>Get Sample FASTA</button>
  </div>
  <div class="button-wrapper">
    <button class="default primary" on:click={buildReference}>Build Reference</button>
  </div>
{:else}
  <div class="button-wrapper">
    <button class="default primary" on:click={resetReference}>Reset</button>
  </div>
{/if}

<style>
  div.status {
    font-size: 0.9rem;
  }
</style>
