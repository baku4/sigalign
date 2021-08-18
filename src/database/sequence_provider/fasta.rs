pub struct FastaProvider<'a> {
    file_path: &'a str
}

enum Extension {
    None,
    Gunzip,
}