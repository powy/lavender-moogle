use std::error::Error;
use std::time::{Instant};
use image::{RgbImage};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
#[allow(unused)]
use rten_tensor::prelude::*;


const DEFAULT_DETECTION_MODEL_PATH: &str = "models/text-detection.rten";
const DEFAULT_REC_MODEL_PATH: &str = "models/text-recognition.rten";

pub fn init_ocr_engine(
    detection_model_path: Option<&str>,
    rec_model_path: Option<&str>,
) -> Result<OcrEngine, Box<dyn Error>> {
    // Use the provided paths or fallback to the default constants
    let detection_model_path = detection_model_path.unwrap_or(DEFAULT_DETECTION_MODEL_PATH);
    let rec_model_path = rec_model_path.unwrap_or(DEFAULT_REC_MODEL_PATH);

    let detection_model = Model::load_file(detection_model_path)?;
    let recognition_model = Model::load_file(rec_model_path)?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    
    Ok(engine)
}

pub fn read_text_from_picture(engine:&OcrEngine, image_input: &RgbImage) -> Result<String, Box<dyn std::error::Error>> {

    let start_time_prepare = Instant::now();
    
    let duration_prepare = start_time_prepare.elapsed();
    println!("Time taken to prepare: {:?}", duration_prepare);
    let start_time_open = Instant::now();
    
    let duration_open = start_time_open.elapsed();
    println!("Time taken to open: {:?}", duration_open);
    let start_time_ocr = Instant::now();

// Debug with image_path
//     let img = image::open(image_path).map(|image| image.into_rgb8())?;

    // Apply standard image pre-processing expected by this library (convert
    // to greyscale, map range to [-0.5, 0.5]).
    let img_source = ImageSource::from_bytes(image_input.as_raw(), image_input.dimensions())?;


    let duration_img_source = start_time_ocr.elapsed();
    println!("Time taken to img_source: {:?}", duration_img_source);
    let start_time_ocr_input = Instant::now();

    let ocr_input = engine.prepare_input(img_source)?;
    
    let duration_ocr_input = start_time_ocr_input.elapsed();
    println!("Time taken to ocr_input: {:?}", duration_ocr_input);
    let start_time_get_text = Instant::now();


    // Detect and recognize text. If you only need the text and don't need any
    // layout information, you can also use `engine.get_text(&ocr_input)`,
    // which returns all the text in an image as a single string.

    // Get oriented bounding boxes of text words in input image.
    let read_text = engine.get_text(&ocr_input);

    
    let duration_get_text = start_time_get_text.elapsed();
    println!("Time taken to get_text: {:?}", duration_get_text);



    let duration_ocr = start_time_ocr.elapsed();
    println!("Time taken to read: {:?}", duration_ocr);

    // println("OCR Result: {}", text.unwrap());
    let output = read_text?;

    Ok(output)

}

