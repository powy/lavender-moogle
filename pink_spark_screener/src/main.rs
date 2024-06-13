use std::thread::sleep;
use std::time::{Duration, Instant};

mod capture;
mod reader;
mod memory;

fn main() {
    println!("Screening for pinky sparkles!");

    // Initialize the OCR engine once
    let engine = reader::init_ocr_engine(None, None).expect("Failed to initialize OCR engine");

    // Initialize the Capturer
    let mut capturer = capture::init_capturer().expect("Failed to initialize capturer");

    // init the memory
    let memory_handle = memory::initialize_or_reset_shared_memory().expect("Failed to initialize memory");

    println!("Init done");

    let interval = Duration::from_millis(3000);

    loop {
        let start_time = Instant::now();

        // Capture the screen and process it
        let capture_start = Instant::now();
        let cropped_image = capture::capture(&mut capturer, Some(49), Some(62)).expect("Failed to capture image");
        let capture_elapsed = capture_start.elapsed();
        println!("cropped_image done");

        // Read the processed image using OCR
        let ocr_start = Instant::now();
        let _text_ocr = reader::read_text_from_picture(&engine, &cropped_image).expect("Failed to read text from picture");
        let ocr_elapsed = ocr_start.elapsed();
        // println!("{}", text_ocr);

        
        let memory_start = Instant::now();
        memory::populate_shared_memory(memory_handle, "abcdefgh").expect("Failed to write to memory");
        let memory_elapsed = memory_start.elapsed();

        // Calculate the elapsed time for the whole loop
        let total_elapsed = start_time.elapsed();

        // Display elapsed times in milliseconds
        println!("Capture time: {} ms", capture_elapsed.as_millis());
        println!("OCR time: {} ms", ocr_elapsed.as_millis());
        println!("Memory time: {} ms", memory_elapsed.as_millis());
        println!("Total elapsed time: {} ms", total_elapsed.as_millis());

        // Ensure the process starts exactly every 3000 milliseconds
        if total_elapsed < interval {
            sleep(interval - total_elapsed);
        }
        
        println!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=");
    }
}
