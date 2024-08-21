use std::sync::{Arc, Mutex};
use std::thread;
use log::{debug, error};

mod types;
mod character_init;
mod spell;
mod server;
mod memory;
mod job_management;
mod rotation_paladin;
mod rotation_gunbreaker;
mod rotation_summoner;

fn main() {
    // Force set the RUST_LOG environment variable
    use std::env;
    env::set_var("RUST_LOG", "debug");  // Change "debug" to the desired log level

    // Initialize the logger
    env_logger::init();

    debug!("Here come the lavender moogles!");

    let spell_file_path = "moogle_spell_list.xlsx";
    let character_details_result = character_init::init_character_details_from_excel(spell_file_path);

    let character_details = match character_details_result {
        Ok(book) => Arc::new(Mutex::new(book)),
        Err(e) => {
            error!("Failed to initialize spell book: {}", e);
            return;
        }
    };
    // Initialize the memory
    memory::initialize_or_reset_shared_memory().expect("Failed to initialize memory");


    // Thread 1: Processing Loop
    let character_details_clone = Arc::clone(&character_details);
    let processing_thread = thread::spawn(move || {
        processing_loop(character_details_clone);
    });

    // Thread 2: JSON Server
    let character_details_clone = Arc::clone(&character_details);
    let server_thread = thread::spawn(move || {
        server::start_json_server(character_details_clone);
    });

    // Wait for both threads to finish
    processing_thread.join().unwrap();
    server_thread.join().unwrap();
}

fn processing_loop(shared_character_details: Arc<Mutex<types::CharacterDetails>>) {
    let interval = std::time::Duration::from_millis(75);

    loop {
        // let current_time = chrono::Local::now();
        // debug!("Current time: {}", current_time.format("%Y-%m-%d %H:%M:%S"));

        // debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        let start_time = std::time::Instant::now();
        
        {
            let mut character_details = shared_character_details.lock().unwrap();
            job_management::apply_current_job_rotation(&mut character_details);
        } // The lock is released here when the scope ends


        let total_elapsed = start_time.elapsed();
        // debug!("Total elapsed time: {} ms", total_elapsed.as_millis());
        // debug!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        if total_elapsed < interval {
            std::thread::sleep(interval - total_elapsed);
        }

    }
}
