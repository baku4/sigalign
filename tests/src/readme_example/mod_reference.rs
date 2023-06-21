#[test]
fn reference_module() {
use sigalign::reference::{
    Reference,
    sequence_storage::in_memory::InMemoryStorage,
    pattern_index::lfi::{Lfi32B2V64, LfiOption},
};

// (1) Define the storage of targets
let mut sequence_storage = InMemoryStorage::new();
sequence_storage.add_target(
    "target_1",
    b"AAAA...AAA",
);
sequence_storage.add_target(
    "target_2",
    b"CCCC...CCC",
);

// (2) Define option for PatternIndex
let pattern_index_option = LfiOption::new(2, 4, true);

// (3) Build reference
let reference = Reference::<Lfi32B2V64, InMemoryStorage>::new(
    sequence_storage,
    pattern_index_option,
).unwrap();

use sigalign::wrapper::DefaultAligner;

let mut aligner = DefaultAligner::new_local(4, 6, 2, 50, 0.2).unwrap();

let result = aligner.align_query(
    &reference,
    b"AA...CC",
);
let result = aligner.align_fasta_file(
    &reference,
    "FASTA_FILE_PATH",
);

let mut sequence_buffer = reference.get_sequence_buffer();
for query in [b"AA...CC", b"GG...TT"] {
    aligner.alignment(
        &reference,
        &mut sequence_buffer,
        query,
    );
}

let mut reference = reference;
reference.set_search_range(vec![0, 1]).unwrap();

use sigalign::reference::extensions::Serialize;
let mut buffer = Vec::new();
reference.save_to(&mut buffer).unwrap();
let reference = Reference::<Lfi32B2V64, InMemoryStorage>::load_from(&buffer[..]).unwrap();

}