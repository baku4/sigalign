<script lang="ts">
  import ToggleButton from "../components/ToggleButton.svelte";
  import { Aligner, AlignerStatus } from "../../wasm/sigalign_demo_wasm";

  export let aligner: Aligner;

  const DEF_PX: number = 4;
  const DEF_PO: number = 6;
  const DEF_PE: number = 2;
  const DEF_ML: number = 100;
  const DEF_MPPL: number = 0.1;
  const DEF_IS_LOCAL: boolean = true;

  let px: number = DEF_PX;
  let po: number = DEF_PO;
  let pe: number = DEF_PE;
  let ml: number = DEF_ML;
  let mppl: number = DEF_MPPL;
  let isLocal: boolean = DEF_IS_LOCAL;

  let isAdvancedOptionOpened: boolean = false;
  let errorMsg: string = null;

  let alignerStatus: AlignerStatus = null;

  function makeAligner() {
    try {
      aligner = new Aligner(isLocal, px, po, pe, ml, mppl);
      alignerStatus = aligner.get_status();
      errorMsg = null;
    } catch (err) {
      errorMsg = err;
    }
  }
  function resetValues() {
    px = DEF_PX;
    po = DEF_PO;
    pe = DEF_PE;
    ml = DEF_ML;
    mppl = DEF_MPPL;
    isLocal = DEF_IS_LOCAL;
  }
  function resetAligner() {
    aligner.drop();
    aligner = null;
    alignerStatus = null;
  }
</script>

<h3 class="header">🛠️ Regulators</h3>
{#if aligner === null}
  <div class="regulators">
    <div class="rtype"><b>Penalty</b></div>
    <div class="regulator">
      <div class="text">Mismatch</div>
      <div class="input"><input bind:value={px} type="number" min="1" max="10" step="1"></div>
    </div>
    <div class="regulator">
      <div class="text">Gap-open</div>
      <div class="input"><input bind:value={po} type="number" min="0" max="10" step="1"></div>
    </div>
    <div class="regulator">
      <div class="text">Gap-extend</div>
      <div class="input"><input bind:value={pe} type="number" min="1" max="10" step="1"></div>
    </div>
  </div>
  <div class="regulators">
    <div class="rtype"><b>Similarity cutoff</b></div>
    <div class="regulator">
      <div class="text">Min. length</div>
      <div class="input"><input bind:value={ml} type="number" min="30" max="1000" step="10"></div>
    </div>
    <div class="regulator">
      <div class="text">Max. penalty per length</div>
      <div class="input"><input bind:value={mppl} type="number" min="0.0001" max="1" step="0.001"></div>
    </div>
  </div>

  <ToggleButton
    bind:toggled={isAdvancedOptionOpened}
    text="Advanced settings"
  />
  {#if isAdvancedOptionOpened}
    <div class="advanced-option">
      🚧 Under the construction.🚧 <br>
    </div>
  {/if}
{:else}
  <div style="margin-top: 1rem;margin-left: 1rem;">Your <span class="highlight">Aligner</span> is ready.</div>
  {#if alignerStatus !== null}
    <div class="status">
      <ul>
        <li><b>Penalty</b></li>
        <ul>
          <li>Mismatch: {alignerStatus.px}</li>
          <li>Gap-open: {alignerStatus.po}</li>
          <li>Gap-extend: {alignerStatus.pe}</li>
        </ul>
        <li><b>Cutoff</b></li>
        <ul>
          <li>Min.length: {alignerStatus.ml}</li>
          <li>Max. penalty per length: {alignerStatus.mppl.toFixed(4)}</li>
        </ul>
        <li><b>Mode:</b> {alignerStatus.is_local ? 'Local' : 'Semi-global'}</li>
        <li><b>Pattern size:</b> {alignerStatus.pattern_size}</li>
      </ul>
    </div>
  {/if}
{/if}

{#if errorMsg !== null}
  {errorMsg}
{/if}
{#if aligner === null}
  <div class="button-wrapper">
    <button class="default primary" on:click={resetValues}>Reset values</button>
  </div>
  <div class="button-wrapper">
    <button class="default primary" on:click={makeAligner}>Make Aligner</button>
  </div>
{:else}
  <div class="button-wrapper">
    <button class="default primary" on:click={resetAligner}>Reset Aligner</button>
  </div>
{/if}

<style>
  div.regulators {
    padding: 1rem 2rem;
  }
  div.rtype {
    font-size: 1.2rem;
    padding: 0.4rem 1rem;
  }
  div.regulator {
    padding: 0.2rem 2rem;
    display: flex;
    align-items: center;
    flex-flow: row;
  }
  div.text {
    display: inline-block;
    width: 15rem;
  }
  div.input {
    display: inline-block;
    width: 5rem;
  }
  div.advanced-option {
    font-size: inherit;
    margin-left: 1rem;
    padding: 1rem 1rem;
  }
  div.status {
    font-size: 0.9rem;
  }
  input {
    width: 100%;
  }
  @media screen and (max-width: 720px) {
    div.rtype {
      text-align: center;
    }
    div.regulator {
      flex-flow: column;
    }
    div.text {
      text-align: center;
    }
    div.input {
      width: 10rem;
    }
  }
</style>