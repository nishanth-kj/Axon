use core::logger;

fn main() {
    logger::init();
    println!("Desktop starting up...");
    core::run_core_logic();
}
