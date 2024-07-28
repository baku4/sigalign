# `SigAlign` for Python

SigAlign is a sequence alignment algorithm. This repository hosts the Python language bindings for SigAlign. The original project can be found [here](https://github.com/baku4/sigalign).

## Requirements

- Python >= 3.10

## Installation

- Via `pip`

    ```bash
    pip install sigalign
    ```

- Manual build: SigAlign relies on `maturin` as a backend. To build manually:

    ```bash
    pip install maturin
    maturin develop
    ```

## Usage Example

### (1) Import SigAlign

```python
from sigalign import Reference, Aligner
```

### (2) Construct `Reference`

```python
# Build Reference object from `iterable` of tuples (label, sequence).
reference = Reference.from_iterable([
    ("target_1", "ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA"),
    ("target_2", "TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"),
])
# Or only sequences
reference = Reference.from_iterable([
    "ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA",
    "TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC",
])
# Bytes can be used instead of strings
reference = Reference.from_iterable([
    b"ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA",
    b"TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC",
])

# FASTA format can be used
reference = Reference.from_fasta(b""">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC""")
# Or from a file
# reference = Reference.from_fasta_file("reference.fasta")
```

#### Check status of `Reference`

```python
print("# Reference Status")
print(f" - Num targets: {reference.num_targets}")
print(f" - Total length: {reference.total_length} bps")
print(f" - Estimated size: {reference.estimated_size / 1024:.2f} KiB")
```

- Output:

    ```text
    # Reference Status
      - Num targets: 2
      - Total length: 140 bps
      - Estimated size: 1.32 KiB
    ```

#### Parse target label and sequence

```python
for target_index in range(reference.num_targets):
    print(f"# Target {target_index}")
    print(f"  - Label: {reference.get_label(target_index)}")
    print(f"  - Sequence: {reference.get_sequence(target_index)}")
```

- Output:

    ```text
    # Target 0
      - Label: target_1
      - Sequence: ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
    # Target 1
      - Label: target_2
      - Sequence: TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC
    ```

#### Save and load

```python
# Save
reference.save_to_file("reference.sigref")

# Load
reference = Reference.load_from_file("reference.sigref")
```

### (3) Initialize `Aligner`

```python
aligner = Aligner(
    4,     # Mismatch penalty
    6,     # Gap-open penalty
    2,     # Gap-extend penalty
    50,    # Minimum length
    0.2,   # Maximum penalty per length
    use_local_mode=True, # Use local alignment (default: True)
    use_limit=None,      # Limit the number of alignments (default: None)
    use_chunk=None,      # Align with chunked query with (chunk size, sliding window size) (default: None)
)
```

#### Check status of `Aligner`

```python
print("# Aligner Status")
print("  - Penalties")
print(f"    - Mismatch penalty: {aligner.px}")
print(f"    - Gap-open penalty: {aligner.po}")
print(f"    - Gap-extend penalty: {aligner.pe}")
print("  - Similarity Cutoffs")
print(f"    - Minimum length: {aligner.minl}")
print(f"    - Maximum penalty per length: {aligner.maxp:.2f}")
print(f"  - Mode is {'Local' if aligner.is_local_mode else 'Semi-global'}")
print(f"    - Max alignments: {'Infinity' if aligner.limitation is None else aligner.limitation}")
print(f"    - Chunk: {aligner.chunk}")
```

- Output:

    ```text
    # Aligner Status
    - Penalties
        - Mismatch penalty: 4
        - Gap-open penalty: 6
        - Gap-extend penalty: 2
    - Similarity Cutoffs
        - Minimum length: 50
        - Maximum penalty per length: 0.20
    - Mode is Local
        - Max alignments: Infinity
        - Chunk: None
    ```

### (4) Perform Alignment

```python
# Align a query str to the reference
query = "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"
results = aligner.align_query(query, reference)

# Or query bytes can be used
query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"
results = aligner.align_query(query, reference, with_label=True) # including label is slightly slower than without label (default: False)

# FASTA (str or bytes) can be used
fasta = b""">query_1
CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"""
results = aligner.align_fasta(
    fasta,
    reference,
)
# Or file can be used:
# results = aligner.align_fasta_file(
#     "path/to/file.fasta",
#     reference,
# )

# FASTQ (str or bytes) can be used:
fastq = b"""@query_1
CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA
+
IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII"""
results = aligner.align_fastq(
    fastq,
    reference,
    with_label=True, # include label in the result (default: False)
    with_reverse_complementary=False, # align both forward and reverse complementary (default: False)
    allow_interrupt=True, # allow interrupting with KeyboardInterrupt (default: False)
)
# Or file can be used:
# results = aligner.align_fastq_file(
#     "path/to/file.fastq",
#     reference,
# )
```

### (5) Display Results

```python
for read_alignment in results:
    print(f"# Query: {read_alignment.read} (is forward: {read_alignment.is_forward})")
    for target_alignment in read_alignment.result:
        print(f"  - Target: {target_alignment.label} (index {target_alignment.index})")
        for idx, alignment in enumerate(target_alignment.alignments):
            print(f"    - Result {idx+1}")
            print(f"      * Penalty: {alignment.penalty}")
            print(f"      * Length: {alignment.length}")
            print(f"      * Query position: {alignment.query_position}")
            print(f"      * Target position: {alignment.target_position}")
```

- Output:

    ```text
    # Query: query_1 (is forward: True)
    - Target: target_1 (index 0)
        - Result 1
        - Penalty: 8
        - Length: 60
        - Query position: (0, 60)
        - Target position: (10, 70)
    - Target: target_2 (index 1)
        - Result 1
        - Penalty: 8
        - Length: 51
        - Query position: (10, 60)
        - Target position: (9, 60)
    ```

#### Convert results to json or dict

```python
import json
json.loads(results.to_json())
```

- Output:

    ```json
    [{'read': 'query_1',
    'is_forward': True,
    'result': [{'index': 0,
        'label': 'target_1',
        'alignments': [{'penalty': 8,
        'length': 60,
        'query_position': [0, 60],
        'target_position': [10, 70],
        'operations': [{'operation': 'Match', 'count': 27},
        {'operation': 'Subst', 'count': 1},
        {'operation': 'Match', 'count': 17},
        {'operation': 'Subst', 'count': 1},
        {'operation': 'Match', 'count': 14}]}]},
    {'index': 1,
        'label': 'target_2',
        'alignments': [{'penalty': 8,
        'length': 51,
        'query_position': [10, 60],
        'target_position': [9, 60],
        'operations': [{'operation': 'Match', 'count': 23},
        {'operation': 'Deletion', 'count': 1},
        {'operation': 'Match', 'count': 27}]}]}]}]
    ```

#### Convert results to a table

```python
import pandas as pd
df = pd.DataFrame(
    results.to_rows(),
    columns = [
        'query_label', 'is_forward',
        'target_index', 'target_label', 'penalty', 'length',
        'query_start', 'query_end', 'target_start', 'target_end', 'operations',
    ],
)
df
```

- Output:

    |   | query_label | is_forward | target_index | target_label | penalty | length | query_start | query_end | target_start | target_end |         CIGAR |
    |--:|------------:|-----------:|-------------:|-------------:|--------:|-------:|------------:|----------:|-------------:|-----------:|--------------:|
    | 0 |     query_1 |       TRUE |            1 |     target_2 |       8 |     51 |          10 |        60 |            9 |         60 | 23=1D27=      |
    | 1 |     query_1 |       TRUE |            0 |     target_1 |       8 |     60 |           0 |        60 |           10 |         70 | 27=1X17=1X14= |

```python
import polars as pl
df = pl.DataFrame(
    results.to_rows(),
    orient="row",
    schema=[
        'query_label', 'is_forward',
        'target_index', 'target_label', 'penalty', 'length',
        'query_start', 'query_end', 'target_start', 'target_end', 'operations',
    ],
)
df
```

- Output:

    | query_label | is_forward | target_index | target_label | penalty | length | query_start | query_end | target_start | target_end |           CIGAR |
    |------------:|-----------:|-------------:|-------------:|--------:|-------:|------------:|----------:|-------------:|-----------:|----------------:|
    |         str |       bool |          i64 |          str |     i64 |    i64 |         i64 |       i64 |          i64 |        i64 |             str |
    |   "query_1" |       true |            1 |   "target_2" |       8 |     51 |          10 |        60 |            9 |         60 |      "23=1D27=" |
    |   "query_1" |       true |            0 |   "target_1" |       8 |     60 |           0 |        60 |           10 |         70 | "27=1X17=1X14=" |

## Additional Information

This Python library provides bindings for the Rust crate `sigalign`. It offers a set of functions sufficient for most common tasks. However, for more customization, using the Rust crate directly is recommended.

### Support

For any questions or issues, please refer to the original project's GitHub [issue tracker](https://github.com/baku4/sigalign/issues).

### License

SigAlign for Python is released under the MIT license.

### Citation

Bahk, K., & Sung, J. (2024). SigAlign: an alignment algorithm guided by explicit similarity criteria. *Nucleic Acids Research*, gkae607.
