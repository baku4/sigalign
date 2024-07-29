use sigalign_read_mapper::Application;

fn main() {
    Application::run().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}
