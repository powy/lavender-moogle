use std::thread::sleep;
use std::time::{Duration, Instant};
use std::env;
use chrono::Local;
use log::{debug, error, info, log_enabled, Level};
use env_logger;

mod memory;

mod types;
mod spell_init;

fn main() {
    // Force set the RUST_LOG environment variable
    env::set_var("RUST_LOG", "debug");  // Change "debug" to the desired log level

    // Initialize the logger
    env_logger::init();

    debug!("Here come the lavender moogles!");

    // Configure the spell book associating job - spell name - spell characteristics and output keys
    let spell_file_path = "moogle_spell_list.xlsx";
    let spell_book = spell_init::init_spell_book_from_excel(spell_file_path)
    // if log_enabled!(Level::Debug) {
    //   spell_init::pretty_print_spell_book(&spell_book);
    // }

    // Initialize the memory
    let _memory_handle = memory::initialize_or_reset_shared_memory().expect("Failed to initialize memory");

    debug!("Init done");

    let interval = Duration::from_millis(2000);

    loop {
        // Print the current time at the start of the loop iteration
        let current_time = Local::now();
        debug!("Current time: {}", current_time.format("%Y-%m-%d %H:%M:%S"));

        debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        let start_time = Instant::now();
        
        debug!("Process...");
        let total_elapsed = start_time.elapsed();
        info!("Total elapsed time: {} ms", total_elapsed.as_millis());
        if total_elapsed < interval {
            sleep(interval - total_elapsed);
        }

        debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
    }
}
