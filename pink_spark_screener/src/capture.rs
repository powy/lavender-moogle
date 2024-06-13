use image::{ImageBuffer, Rgb, RgbImage};
use scrap::{Capturer, Display};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::error::Error;

pub fn init_capturer() -> Result<Capturer, Box<dyn Error>> {
    let display = Display::primary()?;
    let capturer = Capturer::new(display)?;
    Ok(capturer)
}

pub fn capture(capturer: &mut Capturer, kept_width: Option<u32>, kept_height: Option<u32>) -> Result<RgbImage, Box<dyn std::error::Error>> {
    let kept_width = kept_width.unwrap_or(49);
    let kept_height = kept_height.unwrap_or(62);

    let screen_width = capturer.width();
    let mut error_loop_start: Option<Instant> = None;

    loop {
        // Frame capture timing
        let frame_start = Instant::now();
        match capturer.frame() {
            Ok(frame) => {
                let frame_elapsed = frame_start.elapsed();
                println!("Frame capture time: {} ms", frame_elapsed.as_millis());

                // Post-processing timing
                let post_start = Instant::now();
                let mut cropped_image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(kept_width, kept_height);

                for y in 0..kept_height {
                    for x in 0..kept_width {
                        let idx = (y as usize * screen_width + x as usize) * 4;
                        let r = frame[idx];
                        let g = frame[idx + 1];
                        let b = frame[idx + 2];

                        cropped_image.put_pixel(x, y, Rgb([r, g, b]));
                    }
                }

                let post_elapsed = post_start.elapsed();
                println!("Post-processing time: {} ms", post_elapsed.as_millis());

                if let Some(start) = error_loop_start {
                    let error_loop_elapsed = start.elapsed();
                    println!("Time spent in error loop: {} ms", error_loop_elapsed.as_millis());
                }

                return Ok(cropped_image);
            },
            Err(_) => {
                if error_loop_start.is_none() {
                    error_loop_start = Some(Instant::now());
                }
                // Frame not ready, retry after some delay
                // sleep(Duration::new(0, 1_000_000_000 / 240)); // 240 FPS
                sleep(Duration::new(0, 1_000_000)); // 1 ms
            }
        }
    }
}
