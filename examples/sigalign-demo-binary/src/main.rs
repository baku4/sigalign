use sigalign_demo_binary::Application;

fn main() {
    env_logger::init();
    Application::run().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}
