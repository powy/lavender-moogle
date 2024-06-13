use image::{ImageBuffer, Rgba};
use scrap::{Capturer, Display};
use std::time::{Duration, Instant};
use std::{path::Path, thread::sleep};

pub fn capture_and_save() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let screen_width = capturer.width();
    // let screen_height = capturer.height();
    
    let kept_width = 100;
    let kept_height = 100;

    loop {
        let start_capture = Instant::now();
        match capturer.frame() {
            Ok(frame) => {
                let duration_capture = start_capture.elapsed();
                println!("Time taken to capture: {:?}", duration_capture);

                let start_process = Instant::now();

                // Create a buffer for the top-left 100x100 pixels
                let mut cropped_image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(kept_width as u32, kept_height as u32);

                // Copy and process only the top-left 100x100 pixels
                for y in 0..kept_height {
                    for x in 0..kept_width {
                        let idx = (y * screen_width + x) * 4;
                        let r = frame[idx];
                        let g = frame[idx + 1];
                        let b = frame[idx + 2];
                        let a = frame[idx + 3];

                        cropped_image.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
                    }
                }

                let duration_process = start_process.elapsed();
                println!("Time taken to process: {:?}", duration_process);

                let start_save = Instant::now();
                // Save processed image to a file
                let output_path = Path::new("output/screenshot.png");
                cropped_image.save(output_path).expect("Failed to save image");
                let duration_save = start_save.elapsed();
                println!("Time taken to save: {:?}", duration_save);

                println!("Screenshot saved to {:?}", output_path);

                break; // For demonstration, we break after one capture.
            },
            Err(_) => {
                // Frame not ready, retry after some delay
                sleep(Duration::new(0, 1_000_000_000 / 60)); // 60 FPS
            }
        }
    }
}
