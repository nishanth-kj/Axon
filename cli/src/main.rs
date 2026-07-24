use core::logger;

fn main() {
    logger::init();
    println!("CLI starting up...");
    core::run_core_logic();
}
