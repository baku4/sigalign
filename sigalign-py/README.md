# *SigAlign* for python
`Python` language binding of [*`SigAlign`*](https://github.com/baku4/sigalign).
## Requirements
* python >=3.7
## Install
```bash
pip install sigalign
```
## Usage Example
```python
from sigalign import SequenceStorage, Reference, Aligner

# 1. Define `SequenceStorage`
ss = SequenceStorage()
ss.add_record("record_1", "ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA")
ss.add_record("record_2", "TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC")

# 2. Build `Reference`
reference = Reference(
    ss,
    klt_size = 2,
    sas_ratio = 2,
    comp_block = True,
)

# 3. Make `Aligner`
aligner = Aligner(
    4,
    6,
    2,
    50,
    0.2,
    is_local_mode = True,
)

# 4. Perform alignment
query = "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA"
result = aligner.align_query(reference, query)

# 5. Print result
import json
json.loads(result.to_json())

import pandas as pd
df = pd.DataFrame(
    result.to_2d_array(),
    columns = [
        'index', 'label', 'penalty', 'length',
        'qstart', 'qend', 'rstart', 'rend', 'operations',
    ]
)
```
## Build manually
* Use [`maturin`](https://github.com/PyO3/maturin) as backend
    ```bash
    pip install maturin
    maturin develop
    ```