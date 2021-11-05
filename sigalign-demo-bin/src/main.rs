use sigalign_demo_bin::Configuration;

fn main() {
    let config = Configuration::get_matches();
    let result = Configuration::interpret(&config);
}
