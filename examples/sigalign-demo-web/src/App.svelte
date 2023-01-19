<script lang="ts">
  import Callout from './lib/Callout.svelte';
  import TextArea from './lib/TextArea.svelte';
  import { onMount } from 'svelte'
  import init, { Reference, Aligner, say_hello } from './wasm/sigalign_demo_wasm'


  const sample_input_fasta = ">first_record\nAGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCAGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGATACTGTTTCTCCATGCTGCATACACAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAG";
  let input_fasta = sample_input_fasta;

  let input_query = "";

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
    <h1>Welcome!</h1>
    <p>
      <i>SigAlign</i> is a library for pairwise alignment of biological (nucleotide and amino acid) sequences. Pairwise sequence alignment (PSA) is the process of matching two sequences to identify similarity, and it is a critical step in many bioinformatics and computational biology applications. If you are new to sequence alignment, a quick scratch on <a href="https://en.wikipedia.org/wiki/Sequence_alignment" target="_blank" rel="noreferrer">Wikipedia</a> will be helpful.
    </p>
    <p>
      <i>SigAlign</i> is designed to be:
    </p>
    <ul>
      <li><b>💡 Explainable:</b> to escape from “*black box*⬛.”</li>
      <li><b>⚡️ Fast:</b> to solve real-world problems.</li>
      <li><b>🧱 Small and flexible:</b> to be a basic building block for other tools.</li>
    </ul>
    <p>
      We invite you to take a tour of performing PSA through this web app. PSA on this page is powered by <i>SigAlign</i>, built as WebAssembly. No matter what device you access on this page, the alignment proceeds on that device, not a remote Linux server! By step-by-step instructions, you will find how intuitive and fast SigAlign is.
    </p>
    

    <h1 class="header">How does it work?</h1>
    <p>
      First, prepare two structures:
    </p>
    <ul>
      <li><span class="highlight">Reference</span>: Database of multiple target sequences and their index</li>
      <li><span class="highlight">Aligner</span>: Executor for alignment</li>
    </ul>
    <p>
      Then, passing the <span class="highlight">Reference</span> and query sequence to the <span class="highlight">Aligner</span> gives you the alignment result.
      The result can be shown in JSON or table format in this tour.
    </p>


    <h1 class="header">Step 1. Prepare Requirements</h1>
    <h2 class="header">(1) Build <span class="highlight">Reference</span></h2>
    <p>
      Using multiple target sequences is very common in the real world. <i>SigAlign</i> also has the <span class="highlight">Reference</span> structure to access many sequences at one time.
    </p>
    <p>
      Unlike some other algorithms, in <i>SigAlign</i>, the composition of target sequences does not affect the alignment result. In some algorithms, the alignment results for multiple separated references can not be included in the result for one combined reference. On the other hand, in <i>SigAlign</i>, the sum of results of one-to-one alignment for all target sequences in <span class="highlight">Reference</span> is always the same as the result of using the one <span class="highlight">Reference</span>.
    </p>
    <p>
      <i>SigAlign's</i> <span class="highlight">Reference</span> is assumed to be used in a multi-threading environment. <span class="highlight">Reference</span> is an immutable object with no race condition (i.e., read-only) that can be safely shared between threads.
    </p>
    <Callout>
      <span slot="title">
        Structural detail of <span class="highlight">Reference</span>
      </span>
      <span slot="contents">
        🚧Under the construction.🚧
      </span>
    </Callout>
    <p>
      The easiest way to make <span class="highlight">Reference</span> is to use <span class="highlight">Builder</span> structure. When sequences are passed to the <span class="highlight">Builder</span>, it takes care of all other processes (indexing, inferring the type of sequences, etc.).
    </p>
    <div class="subtask">
      <h3 class="header">🛠️ Reference Builder</h3>
      <p>
        In this tour, we will use <a href="https://en.wikipedia.org/wiki/FASTA_format" target="_blank" rel="noreferrer">FASTA</a> formatted string to define multiple sequences to use as a target. We give you sample sequences that you can modify.
      </p>
      <TextArea
        bind:value={input_fasta}
        height_rem={8}
      />
      <button>Advanced settings</button>
      <button>Build Reference</button>
    </div>

    
    
    <h2 class="header">(2) Make <span class="highlight">Aligner</span></h2>
    <p>
      The <span class="highlight">Aligner</span> has a specification for how to align. It performs alignment when query and reference are given, automatically managing the workspace for alignment, which is very small and reusable.
    </p>
    <p>
      You can create <span class="highlight">Aligner</span> by specifying the parameters that regulate the alignment result. In some algorithms, the <span class="highlight">--help</span> command prints over 50 input parameters, of which more than 20 affect the results. And sometimes, it isn't easy to understand the meaning of the parameter only with the help of the <span class="highlight">--help</span> command. Fortunately, <i>SigAlign</i>'s <span class="highlight">Aligner</span> has only five parameters having straightforward definitions.
    </p>
    <ol>
      <li>Penalty</li>
      <ol type="a">
        <li>Mismatch</li>
        <li>Gap-open</li>
        <li>Gap-extend</li>
      </ol>
      <li>Similarity cutoff</li>
      <ol type="a">
        <li>Minimum length (ML)</li>
        <li>Maximum penalty per length (MPpL)</li>
      </ol>
    </ol>
    <p>
      The penalty is a function to calculate similarity. There are three penalties because SigAlign uses the <a href="https://en.wikipedia.org/wiki/Gap_penalty#Affine" target="_blank" rel="noreferrer">gap-affine penalty</a>, widely used to reflect the complexity of the biological sequence.
    </p>
    <p>
      The similarity cutoff is the bound of the result. The similarity in <i>SigAlign</i> is intuitive and straightforward. Alignment with a longer length and smaller penalty per length is more similar. If you are interested in only the results with high similarity, you can perform alignment faster using more strict cutoffs.
    </p>
    <p>
      Basically, <i>SigAlign</i> has no default option. But for convenience, we preset the penalty to 4, 6, and 2. Throughput is very sensitive to cutoffs. For the expected length of input query <i>l</i>, using <i>5√l</i> and <i>0.5/√l</i> for ML and MPpl will show a reasonable speed.
    </p>
    <div class="subtask">
      <h3 class="header">Regulators</h3>
    </div>
    


    <h1 class="header">Step 2. Perform Alignment</h1>
    <p>
      Now you can input the query and press the "Start Alignment" button to get a result. If you're empty-handed, the "Get sample query" button will create an appropriate sequence up to 100bp long.
    </p>
    <p>
      Pressing the start button has a clear meaning. <i>SigAlign</i> outputs the same result as: (1) in order from the optimal (i.e., small penalty) of all alignments, (2) outputs the unique alignment satisfying the cutoffs. This is different from how <i>SigAlign</i> actually works. But it's okay to ignore the detail of the algorithm. You just need to be sure about the result:
    </p>
    <ol>
      <li>If there is a result, the optimal alignment is always included.</li>
      <li>If the optimal alignment does not satisfy the cutoff, there is no result.</li>
    </ol>
    <div class="subtask">
      <h3 class="header">🛠️ Pass the query</h3>
      <p>
      </p>
      <TextArea
        bind:value={input_query}
        height_rem={4}
      />
      <button>Get sample query</button>
      <button>Start Alignment</button>
    </div>


    <h1 class="header">Step 3. View Result</h1>
    <p>
      The default structure of the result includes the following:
    </p>
    <ol>
      <li>Index: an index of the target sequence in the reference</li>
      <li>Alignments: an array of alignments that include</li>
      <ol type="a">
        <li>Penalty: total penalty of alignment</li>
        <li>Length: total length of alignment</li>
        <li>Position: position at query and target</li>
        <li>Operations: a series of &#123;Match, Mismatch (Substitution), Insertion, and Deletion&#125;.</li>
      </ol>
    </ol>
    <p>
      The structure of the result can differ by whether multiple queries is used, or target has a label. In this tour, the result also has a label (our reference stores the label from the IDs of the FASTA string).
    </p>
    <p>
      The raw result returned is structured bytes data. You can transform the result to your desired format. In this tour, we prepared the format of JSON and Table.
    </p>
    <div class="subtask">
      <h3 class="header">🛠️ Result Viewer</h3>
      <button>as JSON</button>
      <button>as Table</button>
    </div>





    <br>
    <br>
    <hr>
    <p>
      Prerequisites:
      1. reference: {have_reference}
      2. aligner: {have_aligner}
    </p>
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
    <hr>
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
  </div>
</main>
