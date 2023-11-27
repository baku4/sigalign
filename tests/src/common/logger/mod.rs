use log::{info, warn, error};
pub fn init_logger() {
    let _ = env_logger::builder()
        .target(env_logger::Target::Stdout)
        .is_test(true)
        .try_init();
}