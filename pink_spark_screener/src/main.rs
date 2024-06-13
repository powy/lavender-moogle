use std::thread::sleep;
use std::time::{Duration, Instant};
use chrono::Local;

mod capture;
mod reader;
mod memory;

fn debug_print(is_debug: bool, message: &str) {
    if is_debug {
        println!("{}", message);
    }
}

fn main() {
    let is_debug = true;

    debug_print(is_debug, "Screening for pinky sparkles!");

    // Initialize the OCR engine once
    let engine = reader::init_ocr_engine(None, None).expect("Failed to initialize OCR engine");

    // Initialize the Capturer
    let mut capturer = capture::init_capturer().expect("Failed to initialize capturer");

    // Initialize the memory
    let memory_handle = memory::initialize_or_reset_shared_memory().expect("Failed to initialize memory");

    debug_print(is_debug, "Init done");

    let interval = Duration::from_millis(3000);

    loop {
        // Print the current time at the start of the loop iteration
        let current_time = Local::now();
        debug_print(is_debug, &format!("Current time: {}", current_time.format("%Y-%m-%d %H:%M:%S")));

        debug_print(is_debug, "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
        let start_time = Instant::now();

        // Capture the screen and process it
        let capture_start = Instant::now();
        match capture::capture(&mut capturer, Some(49), Some(62)) {
            Ok(cropped_image) => {
                let capture_elapsed = capture_start.elapsed();
                debug_print(is_debug, "cropped_image done");

                // Read the processed image using OCR
                let ocr_start = Instant::now();
                let _text_ocr = reader::read_text_from_picture(&engine, &cropped_image).expect("Failed to read text from picture");
                let ocr_elapsed = ocr_start.elapsed();
                // debug_print(is_debug, &text_ocr);

                let memory_start = Instant::now();
                memory::populate_shared_memory(memory_handle, "abcdefgh").expect("Failed to write to memory");
                let memory_elapsed = memory_start.elapsed();

                // Calculate the elapsed time for the whole loop
                let total_elapsed = start_time.elapsed();

                // Display elapsed times in milliseconds
                debug_print(is_debug, &format!("Capture time: {} ms", capture_elapsed.as_millis()));
                debug_print(is_debug, &format!("OCR time: {} ms", ocr_elapsed.as_millis()));
                debug_print(is_debug, &format!("Memory time: {} ms", memory_elapsed.as_millis()));
                debug_print(is_debug, &format!("Total elapsed time: {} ms", total_elapsed.as_millis()));

                // Ensure the process starts exactly every 3000 milliseconds
                if total_elapsed < interval {
                    sleep(interval - total_elapsed);
                }
            },
            Err(e) => {
                debug_print(is_debug, &format!("Failed to capture image: {}", e));
                // Reinitialize the capturer and retry
                debug_print(is_debug, "Reinitializing the capturer...");
                match capture::init_capturer() {
                    Ok(new_capturer) => {
                        capturer = new_capturer;
                        debug_print(is_debug, "Reinitialization successful.");
                    },
                    Err(init_err) => {
                        debug_print(is_debug, &format!("Failed to reinitialize the capturer: {}", init_err));
                        // Sleep for a while before retrying to avoid busy looping
                        sleep(Duration::from_secs(1));
                    }
                }
            }
        }

        debug_print(is_debug, "-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
    }
}
