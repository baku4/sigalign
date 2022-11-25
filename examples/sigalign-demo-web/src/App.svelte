<script lang="ts">
  import svelteLogo from './assets/svelte.svg'
  import Counter from './lib/Counter.svelte'
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
  <div class="title">
    <h1>SigAlign Demo App</h1>
    <p>
      This web application uses SigAlign built with WebAssembly. No matter what device you access this page, the alignment proceeds on your own machine!
    </p>
  </div>

  <div>
    <h2>How it works?</h2>
    <p>
      You need two structures: <span>(1) Reference</span> and <span>(2) Aligner</span>.
    </p>
  </div>
  <div>
    <h2>Pepare Reference</h2>
    <p>
      Reference is a database for multiple alignment target sequences and their index.
    </p>
    <h3>Make Sequence Storage</h3>
    <p>
      SigAlign's Reference is agnostic to storage of sequecnes. In this case, we will use InMemoryStorage.
    </p>
    <h3>Pass Storage to ReferenceBuilder</h3>
    <p>
      contents
    </p>
    <div>
      <button on:click={makeReference}>Make Reference</button>
      <button on:click={resetReference}>Reset Reference</button>
      have reference?: {have_reference}
    </div>
  </div>
  <div>
    <h2>Pepare Aligner</h2>
    <p>
      Aligner has alignment condition and performs alignment on its internal working space.
    </p>
    <p>
      Reference is a database for multiple alignment target sequences and their index.
    </p>
    <h3>Make Sequence Storage</h3>
    <p>
      SigAlign's Reference is agnostic to storage of sequecnes. In this case, we will use InMemoryStorage.
    </p>
    <h3>Pass Storage to ReferenceBuilder</h3>
    <p>
      contents
    </p>
    <div>
      <button on:click={makeAligner}>Make Aligner</button>
      <button on:click={resetAligner}>Reset Aligner</button>
      have aligner?: {have_aligner}
    </div>
  </div>
  <div>
    
  </div>
  
  <div>
    <button on:click={doAlignment}>Alignment</button>
  </div>
  <div>
    Result: {result}
  </div>

  
  
  <div>
    <h2>Prepare Aligner</h2>
    
  </div>
  <div>
    <h2>Perform Alignment</h2>
    <p>
      Aligner has alignment condition and performs alignment on its internal working space.<br/>
    </p>
  </div>

  <div>
    <a href="https://vitejs.dev" target="_blank" rel="noreferrer"> 
      <img src="/vite.svg" class="logo" alt="Vite Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank" rel="noreferrer"> 
      <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
    </a>
  </div>
  <h1>Vite + Svelte</h1>

  <div class="card">
    <Counter />
  </div>

  <p>
    Check out <a href="https://github.com/sveltejs/kit#readme" target="_blank" rel="noreferrer">SvelteKit</a>, the official Svelte app framework powered by Vite!
  </p>

  <p class="read-the-docs">
    Click on the Vite and Svelte logos to learn more
  </p>
</main>

<style>
  span {
    background-color: lightpink;
  }
  /* .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
  }
  .logo:hover {
    filter: drop-shadow(0 0 2em #646cffaa);
  }
  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00aa);
  }
  .read-the-docs {
    color: #888;
  } */
</style>