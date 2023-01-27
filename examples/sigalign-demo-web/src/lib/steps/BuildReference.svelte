<script lang="ts">
  import TextArea from "../components/TextArea.svelte";
  import ToggleButton from "../components/ToggleButton.svelte";
  import {
    type Reference,
    type ReferenceStatus,
    get_sample_target_as_fasta_string,
  } from "../../wasm/sigalign_demo_wasm";

  type BuildFnType = (
    klt: number | undefined, sasr: number | undefined, use_128_bwt: boolean | undefined, fasta: string
  ) => Promise<Reference>;

  export let reference: Reference;
  export let buildRefFn: BuildFnType;

  let fasta: string = "";

  let isAdvancedOptionOpened: boolean = false;
  let useDefKlt: boolean = true;
  let useDefSasr: boolean = true;
  let useDefBwt: boolean = true;

  let klt: number = null;
  let sasr: number = null;
  let use128Bwt: boolean = null;
  
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
    if (useDefKlt) {
      klt = null;
    }
    if (useDefSasr) {
      sasr = null;
    }
    if (useDefBwt) {
      use128Bwt = null;
    }
    let promise = buildRefFn(klt, sasr, use128Bwt, fasta);
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
    klt = null;
    sasr = null;
    use128Bwt = null;
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
  
  <ToggleButton
    bind:toggled={isAdvancedOptionOpened}
    text="Advanced settings"
  />

  {#if isAdvancedOptionOpened}
    <div class="advanced-option">
      🚧 Under the construction.🚧 <br>
      You can control the compression level of index.
    </div>
  {/if}
{:else}
  <div style="margin-top: 1rem;margin-left: 1rem;">Your <span class="highlight">Reference</span> is ready.
    {#if referenceStatus !== null}
      <div class="status">
        <ul>
          <li><b>Total records:</b> #{referenceStatus.total_records}</li>
          <li><b>Type of sequence:</b>
            {#if referenceStatus.is_nucleotide}
            Nucleotide
            {:else}
            Amino acid
            {/if}
            {#if referenceStatus.have_noise}
            with noise
            {/if}
            &#123;{referenceStatus.supported_sequences}&#125;
          </li>
          <li><b>Compression level of index</b></li>
          <ul>
            <li>Lookup table <i>kmer</i> size: {referenceStatus.klt}</li>
            <li>Suffix array sampling ratio: {referenceStatus.sasr}</li>
            <li>BWT block size: {referenceStatus.bwt_block_size}</li>
          </ul>
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
  div.advanced-option {
    font-size: inherit;
    margin-left: 1rem;
    padding: 1rem 1rem;
  }
  div.status {
    font-size: 0.9rem;
  }
</style>
