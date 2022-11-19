# Examples

## `sigalign-demo-aligner`
The binary implementation of *SigAlign*
* build
    ```bash
    cargo build -p sigalign-demo-aligner --release
    ```
* get `help` about sub commands
    ```bash
    sigalign-demo-aligner
    ```

## `sigalign-py-in-jupyter`
The jupyter notebook (or lab) example of using python binding of *SigAlign*.
* install
    ```bash
    pip install sigalign
    ```
* import
    ```python
    from sigalign import SequenceStorage, Reference, Aligner

    # ...

    result = aligner.align_query(reference, query)
    ```
