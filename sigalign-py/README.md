# SigAlign for Python

**SigAlign** is a sequence alignment algorithm. This repository hosts the Python language bindings for SigAlign. The original project can be found [here](https://github.com/baku4/sigalign).

## Requirements

- Python >= 3.7

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
reference = Reference(
    targets = [
        ("target_1", "ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA"),
        ("target_2", "TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"),
    ], indexing_option= {
        "sasr": 2,      # Suffix array sampling ratio
        "lts": 10_000,  # Lookup table size (Bytes)
    },
)
```

#### Alternatively, build from FASTA file

```python
fasta_file_path = "./YOUR_REFERENCE.fa"
reference = Reference.from_fasta_file(fasta_file_path)
```

#### Check status of `Reference`

```python
print("# Reference Status")
print(f" - Num_targets: {reference.num_targets}")
print(f" - Estimated_total_bytes: {reference.estimated_size / 1024:.2f} KiB")
```

- Output:

    ```text
    # Reference Status
    - Num_targets: 2
    - Estimated_total_bytes: 31.44 KiB
    ```

### (3) Initialize `Aligner`

```python
aligner = Aligner(
    4,     # Mismatch penalty
    6,     # Gap-open penalty
    2,     # Gap-extend penalty
    50,    # Minimum length
    0.2,   # Maximum penalty per length
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
print(f"    - Minimum length: {aligner.ml}")
print(f"    - Maximum penalty per length: {aligner.mpl:.1f}")
print(f"  - Mode is {'Local' if aligner.is_local_mode else 'Semi-global'}")
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
        - Maximum penalty per length: 0.2
    - Mode is Local
    ```

### (4) Execute Alignment

```python
query = "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"

results = aligner.align_query(reference, query)
```

#### Alternatively, use FASTA file

```python
fasta_file_path = "./YOUR_QUERY.fa"
result = aligner.align_fasta_file(
    reference,
    fasta_file_path,
)
```

### (5) Display Results

```python
for target_result in results:
    print(f"# Target index: {target_result.index}")
    for idx, alignment in enumerate(target_result.alignments):
        print(f"  - Result: {idx+1}")
        print(f"    - Penalty: {alignment.penalty}")
        print(f"    - Length: {alignment.length}")
        print(f"    - Query position: {alignment.query_position}")
        print(f"    - Target position: {alignment.target_position}")
```

- Output:

    ```text
    # Target index: 0
    - Result: 1
        - Penalty: 8
        - Length: 60
        - Query position: (0, 60)
        - Target position: (10, 70)
    # Target index: 1
    - Result: 1
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

    ```text
    [{'index': 0,
    'label': 'target_1',
    'alignments': [{'penalty': 8,
        'length': 60,
        'query_position': [0, 60],
        'target_position': [10, 70],
        'operations': [{'case': 'M', 'count': 27},
        {'case': 'S', 'count': 1},
        {'case': 'M', 'count': 17},
        {'case': 'S', 'count': 1},
        {'case': 'M', 'count': 14}]}]},
    {'index': 1,
    'label': 'target_2',
    'alignments': [{'penalty': 8,
        'length': 51,
        'query_position': [10, 60],
        'target_position': [9, 60],
        'operations': [{'case': 'M', 'count': 23},
        {'case': 'I', 'count': 1},
        {'case': 'M', 'count': 27}]}]}]
    ```

#### Convert results to a table

```python
import pandas as pd
df = pd.DataFrame(
    results.to_table(),
    columns = [
        'index', 'label', 'penalty', 'length',
        'qstart', 'qend', 'tstart', 'tend', 'operations',
    ]
)
df
```

- Output:

    | index | label   | penalty | length | qstart | qend | tstart | tend | operations   |
    |-------|---------|---------|--------|--------|------|--------|------|--------------|
    | 0     | target_1| 8       | 60     | 0      | 60   | 10     | 70   | M27S1M17S1M14|
    | 1     | target_2| 8       | 51     | 10     | 60   | 9      | 60   | M23I1M27     |

## Additional Information

This python library binds the default wrappers of the `Aligner` and `Reference` structs from the original library. It provides a set of functions sufficient for most common tasks. However, if you wish to customize or optimize the `Aligner` and `Reference`, you can define your own versions and use them with Foreign Function Interface (FFI) in Python.

## Support

For any questions or issues, please refer to the original project's GitHub [issue tracker](https://github.com/baku4/sigalign/issues).

## License

SigAlign for Python is released under the MIT license. For more details, see [LICENSE](https://github.com/baku4/sigalign/LICENSE).
