use sigalign_demo_aligner::Configuration;

fn main() {
    let config = Configuration::get_matches();
    let result = Configuration::interpret(&config);
}
