# *SigAlign*'s demo binary implementation
## Build
```bash
cargo build -p sigalign-demo-aligner --release
```
## Run `help` commands to get usage
* `sigalign-demo-aligner --help`
    ```
    Binary demo implementation

    Usage: sigalign-demo-aligner <COMMAND>

    Commands:
    reference  Generate reference file
    alignment  Alignment with FASTA file
    help       Print this message or the help of the given subcommand(s)

    Options:
    -h, --help     Print help information
    -V, --version  Print version information
    ```
* `sigalign-demo-aligner reference --help`
    ```
    Generate reference file

    Usage: sigalign-demo-aligner reference [OPTIONS] --input <FILE> --output <FILE>

    Options:
    -i, --input <FILE>   Input FASTA path
    -o, --output <FILE>  Output reference path
    -w, --overwrite      Overwrite output reference file
    -r, --reverse        Use reverse complementary sequence (for nucleotide)
    -c, --cpb            Use higher compressed (128) Bwt block
    -s, --ssr <INT>      Suffix array sampling ratio
    -k, --klt <INT>      Kmer size for count array lookup table
    -d, --divide <INT>   Split by sequence length
    -h, --help           Print help information
    -V, --version        Print version information
    ```
* `sigalign-demo-aligner alignment --help`
    ```
    Alignment with FASTA file

    Usage: sigalign-demo-aligner alignment --input <FILE> --reference <FILE> --output <FILE> --penalties <INT> <INT> <INT> --cutoffs <INT> <FLOAT> <--semiglobal|--local>

    Options:
    -s, --semiglobal                   Use semi-global algorithm
    -l, --local                        Use local algorithm
    -i, --input <FILE>                 Input query FASTA path
    -r, --reference <FILE>             SigAlign reference file
    -o, --output <FILE>                Output json path without extension. Output will be saved to {output}.{ref_num}.json
    -p, --penalties <INT> <INT> <INT>  Mismatch, Gap-open and Gap-extend penalties
    -c, --cutoffs <INT> <FLOAT>        Minimum aligned length and maximum penalty per length
    -h, --help                         Print help information
    -V, --version                      Print version information
    ```