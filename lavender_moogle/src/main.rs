use std::sync::{Arc, Mutex};
use std::thread;
use log::{debug, error};

mod types;
mod spell_init;
mod spell_actions;
mod server;
mod memory;
mod paladin_rotation;

fn main() {
    // Force set the RUST_LOG environment variable
    use std::env;
    env::set_var("RUST_LOG", "debug");  // Change "debug" to the desired log level

    // Initialize the logger
    env_logger::init();

    debug!("Here come the lavender moogles!");

    let spell_file_path = "moogle_spell_list.xlsx";
    let spell_book_result = spell_init::init_spell_book_from_excel(spell_file_path);

    let spell_book = match spell_book_result {
        Ok(book) => Arc::new(Mutex::new(book)),
        Err(e) => {
            error!("Failed to initialize spell book: {}", e);
            return;
        }
    };
    // Initialize the memory
    memory::initialize_or_reset_shared_memory().expect("Failed to initialize memory");


    // Thread 1: Processing Loop
    let spell_book_clone = Arc::clone(&spell_book);
    let processing_thread = thread::spawn(move || {
        processing_loop(spell_book_clone);
    });

    // Thread 2: JSON Server
    let spell_book_clone = Arc::clone(&spell_book);
    let server_thread = thread::spawn(move || {
        server::start_json_server(spell_book_clone);
    });

    // Wait for both threads to finish
    processing_thread.join().unwrap();
    server_thread.join().unwrap();
}

fn processing_loop(shared_spell_book: Arc<Mutex<types::SpellBook>>) {
    let interval = std::time::Duration::from_millis(100);

    loop {
        let current_time = chrono::Local::now();
        // debug!("Current time: {}", current_time.format("%Y-%m-%d %H:%M:%S"));

        // debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        let start_time = std::time::Instant::now();
        
        {
            let spell_book = shared_spell_book.lock().unwrap();
            // Process spell_book here
            // debug!("Updating next spell ...");
            // Add more processing logic here as needed
        }

        let total_elapsed = start_time.elapsed();
        // debug!("Total elapsed time: {} ms", total_elapsed.as_millis());
        // debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        if total_elapsed < interval {
            std::thread::sleep(interval - total_elapsed);
        }

    }
}
