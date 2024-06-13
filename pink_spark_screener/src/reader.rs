use std::error::Error;
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

    // Load saved models from the ocrs project
    let detection_model = Model::load_file(detection_model_path)?;
    let recognition_model = Model::load_file(rec_model_path)?;

    // Create the OCR engine wich will be reused at each iteration
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    
    Ok(engine)
}

pub fn read_text_from_picture(engine:&OcrEngine, image_input: &RgbImage) -> Result<String, Box<dyn std::error::Error>> {

    // Debug with image_path
    //     let img = image::open(image_path).map(|image| image.into_rgb8())?;

    // Apply standard image pre-processing expected by this library (convert
    // to greyscale, map range to [-0.5, 0.5]).
    let img_source = ImageSource::from_bytes(image_input.as_raw(), image_input.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    // Detect and recognize text. If you only need the text and don't need any
    // layout information, you can also use `engine.get_text(&ocr_input)`,
    // which returns all the text in an image as a single string.
    // Get oriented bounding boxes of text words in input image.
    let read_text = engine.get_text(&ocr_input);

    let output = read_text?;

    Ok(output)

}

