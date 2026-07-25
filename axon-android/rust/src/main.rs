use axon_core::logger;

fn main() {
    logger::init();
    println!("Android Rust starting up...");
    axon_core::run_core_logic();
}
