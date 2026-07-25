use axon_core::logger;

fn main() {
    logger::init();
    println!("Desktop starting up...");
    axon_core::run_core_logic();
}
