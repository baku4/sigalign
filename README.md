<div align="center">
	<h1><code>SigAlign</code></h1>
	<p>
		A <b>Si</b>milarity-<b>G</b>uided Pairwise <b>Align</b>ment Algorithm
	</p>
	<p>
		<a href="https://github.com/baku4/sigalign/" target="_blank"><img alt="License" src="https://img.shields.io/github/license/baku4/sigalign?style=flat-square"></a>
		<a href="https://crates.io/crates/sigalign/" target="_blank"><img alt="Crates.io" src="https://img.shields.io/crates/v/sigalign.svg?style=flat-square"></a>
		<a href="https://docs.rs/sigalign/latest/"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
		<a href="https://pypi.org/project/sigalign/" target="_blank"><img alt="PyPI" src="https://img.shields.io/pypi/v/sigalign?style=flat-square"></a>
	</p>
</div>

## Key Features

### Simplified Parameters

- **Gap-Affine Penalties:** SigAlign incorporates a gap-affine penalty-based scoring scheme that promotes continuous gaps to reflect the complexity of biological sequences. The gap-affine penalty system is composed of:
	1. Mismatch Penalty
	2. Gap-Open Penalty
	3. Gap-Extend Penalty
- **Similarity Cutoffs:** Defining a clear boundary for alignment results, SigAlign uses two key criteria:
	1. Minimum Length (ML)
	2. Maximum Penalty per Length (MPL)

### Clear Boundary of Results

- If there is no alignment result, the global optimum does not satisfy the cutoff.
- If there is an alignment result, the global optimum must be included in the alignment result.
- The results include all alignment which is not overlapped with more optimal alignment.

## Aims

SigAlign is appropriate for the task of
- quickly picking out similar alignments, rather than quantifying similarity or finding global optimum to also receive results for alignment with low similarity.
- defining boundaries of results clearly, so when describing a result, instead of saying "I used this algorithm", results have their own semantics.

## Quick Start

### For `Rust` developer
- As a Rust library, SigAlign can take advantage of the most abundant features in Rust. SigAlign is a package registered in `crate.io` and can be added using `cargo` in the project directory:
	`cargo add sigalign`
- Example for Quick Scratch
```rust
use sigalign::wrapper::{
    DefaultAligner,
    DefaultReference,
};

// (1) Build `Reference`
let fasta =
br#">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = DefaultReference::from_fasta_bytes(fasta).unwrap();

// (2) Instantiate `Aligner`
let mut aligner = DefaultAligner::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();

// (3) Perform alignment
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align_query(&reference, query).unwrap();
println!("{:#?}", result);
```
- The detailed features and usage are in API documentation of crate (https://docs.rs/sigalign/).

### For `Python` developer
- If you're a Python developer, you can make use of the Python bindings for SigAlign. You can use the `pip` the python package manager to install the package:
`pip install sigalign`
- Here is a quick start example:
```python
from sigalign import Reference, Aligner

# (1) Construct `Reference`
reference = Reference.from_fasta_file("./YOUR_REFERENCE.fa")

# (2) Initialize `Aligner`
aligner = Aligner(4, 6, 2, 50, 0.2)

# (3) Execute Alignment
query = "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"
results = aligner.align_query(reference, query)

# (4) Display Results
for target_result in results:
    print(f"# Target index: {target_result.index}")
    for idx, alignment in enumerate(target_result.alignments):
        print(f"  - Result: {idx+1}")
        print(f"    - Penalty: {alignment.penalty}")
        print(f"    - Length: {alignment.length}")
        print(f"    - Query position: {alignment.query_position}")
        print(f"    - Target position: {alignment.target_position}")
```
- For a detailed guide on how to use SigAlign in Python including a more comprehensive tutorial, please refer to the `sigalign-py` subdirectory [README](https://github.com/baku4/sigalign/tree/main/sigalign-py/README.md).

### For `Web` developer

- SigAlign offers a WebAssembly (WASM) build, opening up the potential for web-based applications. While it is not currently available through package managers such as `npm`, plans for web support are in the pipeline.
- An exemplary WASM implementation can be found within the `example` directory. Below is a TypeScript example showcasing SigAlign's application via this WASM wrapper:

```ts
import init, { Reference, Aligner, type AlignmentResult } from '../wasm/sigalign_demo_wasm';

async function run() {
    await init();

    // (1) Construct `Reference`
    const fasta: string = `>target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC`;
    
    const reference: Reference = await Reference.build(fasta);
    
    // (2) Initialize `Aligner`
    const aligner: Aligner = new Aligner(
        4,    // Mismatch penalty
        6,    // Gap-open penalty
        2,    // Gap-extend penalty
        50,   // Minimum aligned length
        0.2,  // Maximum penalty per length
    );

    // (3) Execute Alignment
    const query: string = "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
    const result: AlignmentResult = await aligner.alignment(query, reference);

    // (4) Parse and Display Results
    const parsedJsonObj = JSON.parse(result.to_json());
    console.log(parsedJsonObj);
}

run();
```
- To gain further insight into web-based implementation of SigAlign, visit the SigAlign [tour page](https://baku4.github.io/sigalign/). This page utilizes the WASM wrapper exemplified above.

## License

SigAlign is released under the MIT License. Please refer to the [LICENSE](https://github.com/baku4/sigalign/blob/main/LICENSE) file in the repository for the full text.