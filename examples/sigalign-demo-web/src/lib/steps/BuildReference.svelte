<script lang="ts">
  import TextArea from "../components/TextArea.svelte";
  import ToggleButton from "../components/ToggleButton.svelte";
  import type { Reference, ReferenceStatus } from "../../wasm/sigalign_demo_wasm";

  type BuildFnType = (
    klt: number | undefined, sasr: number | undefined, use_128_bwt: boolean | undefined, fasta: string
  ) => Promise<Reference>;

  export let reference: Reference;
  export let fasta: string = "";
  export let buildRefFn: BuildFnType;
  
  let isAdvancedOptionOpened: boolean = false;
  let useDefKlt: boolean = true;
  let useDefSasr: boolean = true;
  let useDefBwt: boolean = true;

  let klt: number = null;
  let sasr: number = null;
  let use128Bwt: boolean = null;
  
  let errorMsg: string = null;

  let referenceStatus: ReferenceStatus = null;

  function buildReference() {
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
    In this tour, we will use <a href="https://en.wikipedia.org/wiki/FASTA_format" target="_blank" rel="noreferrer">FASTA</a> formatted string to define multiple sequences to use as a target. We give you sample sequences that you can modify.
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
    </div>
    <!-- You can control the compression level of index.
      Lookup table kmer size
      <input type=checkbox bind:checked={useDefKlt}> use -->
  {/if}
{:else}
  <div style="margin-top: 1rem;margin-left: 1rem;"><b><i>Reference</i></b> is ready.</div>
  {#if referenceStatus !== null}
    <ul>
      <li>Total records: # {referenceStatus.total_records}</li>
      <li>Type of sequence:
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
      <li>Compression level of index</li>
      <ul>
        <li>Lookup table <i>kmer</i> size: {referenceStatus.klt}</li>
        <li>Suffix array sampling ratio: {referenceStatus.sasr}</li>
        <li>BWT block size: {referenceStatus.bwt_block_size}</li>
      </ul>
      <li>Est. byte size:
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
  {/if}
{/if}

<!-- Controller -->
{#if errorMsg !== null}
  <div class="button-wrapper">
    {errorMsg}
  </div>
{/if}
{#if reference === null}
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
</style>
