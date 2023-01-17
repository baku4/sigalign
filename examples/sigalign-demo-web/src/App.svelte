<script lang="ts">
  import Callout from './lib/Callout.svelte';
  import { onMount } from 'svelte'
  import init, { Reference, Aligner, say_hello } from './wasm/sigalign_demo_wasm'

  let result = "None";

  // let reference;
  // let aligner;
  onMount(async () => {
    init();
  })

  // const wasm = import('./wasm/sigalign_demo_wasm');
  // const reference: Promise<Reference> = wasm
  //   .then((value) => {
  //     value.default
  //     return new value.Reference();
  //   });
  
  let sequence_storage = [];

  let aligner: Aligner;
  let reference: Reference;
  // let res = aligner.align(reference, "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA");

  $: have_aligner = !(aligner === undefined);
  $: have_reference = !(reference === undefined);

  function makeAligner() {
    aligner = Aligner.new_test();
  }
  function resetAligner() {
    aligner = undefined;
  }
  function makeReference() {
    reference = Reference.new_test();
  }
  function resetReference() {
    reference = undefined;
  }
  function doAlignment() {
    let res = aligner.align(reference, "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA");
    result = res;
  }
</script>

<main>
  <div id="title">
    <div class="title_content">
      <h1>Tour🚀 of <i>SigAlign</i></h1>
      with simple web app for pairwise alignment
    </div>
  </div>

  <div class="content">
    <h2>Welcome!</h2>
    <p>
      This web application performs pairwise sequence alignment using <span class="highlight"><i>SigAlign</i></span> built to <span class="highlight"><i>WebAssembly</i></span>. No matter what device you access this page, <b>the alignment proceeds on your own machine.</b>
    </p>
    <Callout>
      <span slot="title">
        What is "Alignment"?
      </span>
      <span slot="contents">
        Alignment is to find optimal relationship between two sequences. This is core task for many bioinformatics processes. If you are new to biological sequence alignment, a quick scratch on <a href="https://en.wikipedia.org/wiki/Sequence_alignment" target="_blank" rel="noreferrer">Wikipedia</a> will be helpful.
      </span>
    </Callout>
    <p>
      <span class="highlight"><i>SigAlign</i></span> supports nucleotide and amino acid sequence.
    </p>
    
    <h2 class="header">How it works?</h2>
    <p>
      First, prepare two structures:
    </p>
    <ul>
      <li><span class="highlight">Reference</span> - Database for multiple alignment target sequences.</li>
      <li><span class="highlight">Aligner</span> - Recipe and workspace for alignment.</li>
    </ul>
    Then, pass the (1) query sequence and (2) <span class="highlight">Reference</span> to the <span class="highlight">Aligner</span>.

    <h2 class="header">Build <span class="highlight">Reference</span></h2>
    <p>
      The easiest way to build <span class="highlight">Reference</span> is pass the <span class="highlight">SequenceStorage</span> which is a set of target sequences to <span class="highlight">Reference</span> builder.
      In this app, we can use <i>Fasta</i> formatted string to make  <span class="highlight">SequenceStorage</span>.
      The builder can automatically generate <span class="highlight">Reference</span>, but additional options for compression level about the index of sequences can be defined. Compression level does not affect the alignment result, so can be changed fearlessly.
    </p>
    <Callout>
      <span slot="title">
        Why need <span class="highlight">SequenceStorage</span>?
      </span>
      <span slot="contents">
        In <span class="highlight"><i>SigAlign</i></span>, <span class="highlight">Reference</span> is agnostic to <span class="highlight">SequenceStorage</span>. In this app, we will use <span class="highlight">InMemoryStorage</span> that stores sequences to main memory.
      </span>
    </Callout>
    <div class="subtask">
      <h3>1. <span class="highlight">SequenceStorage</span></h3>
      

      <button class="default">load sample data</button><button>load from file (.fasta)</button>
      <textarea>test</textarea>


      <h3>2. Compression level</h3>
      The compression level does not affect the results at all, only related to the speed of the algorithm.
    </div>
    
    <h2 class="header">Make <span class="highlight">Aligner</span></h2>
    <p>
      Aligner has alignment condition and performs alignment on its internal working space.
    </p>
    <div class="subtask">
      <h3>1. Select mode</h3>
      Local mode, semi-global mode

      <h3>2. Scoring function</h3>
      Scoring function

      <h3>3. Similarity cutoff</h3>
      Similarity cutoff
    </div>
    
    <h2 class="header">Start alignment!</h2>
    <p>
      Prerequisites:
      1. reference: {have_reference}
      2. aligner: {have_aligner}
    </p>

    <hr>
    <div>
      <button on:click={makeReference}>Make Reference</button>
      <button on:click={resetReference}>Reset Reference</button>
      have reference?: {have_reference}
    </div>
    <div>
      <button on:click={makeAligner}>Make Aligner</button>
      <button on:click={resetAligner}>Reset Aligner</button>
      have aligner?: {have_aligner}
    </div>
    <div>
      <button on:click={doAlignment}>Alignment</button>
    </div>
    <div>
      Result: {result}
    </div>
  </div>
</main>
