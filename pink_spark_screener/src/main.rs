use std::thread::sleep;
use std::time::Duration;

mod capture;
mod reader;

fn main() {
    println!("Hello, World!");

    // Add a delay of 5 seconds
    sleep(Duration::from_secs(5));

    capture::capture_and_save();

    // Read the processed image using OCR
    // let image_path = "input/train.png";
    // let image_path = "input/small.png";
    let image_path = "output/screenshot.png";

    reader::init_ocr_engine(None, None).unwrap();

    let text_ocr = reader::read_picture_from_path(image_path);
    println!("OCR Result: {}", text_ocr.unwrap());
}
