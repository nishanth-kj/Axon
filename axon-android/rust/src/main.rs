use core::logger;

fn main() {
    logger::init();
    println!("Android Rust starting up...");
    core::run_core_logic();
}
