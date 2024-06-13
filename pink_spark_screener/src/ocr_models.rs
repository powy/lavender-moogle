use std::error::Error;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use rten_tensor::prelude::*;


// Assuming `Model` and `OcrEngine` are defined elsewhere
struct OcrModels {
    detection_model: Model,
    recognition_model: Model,
}

impl OcrModels {
    fn new(detection_model_path: &str, rec_model_path: &str) -> Result<Self, Box<dyn Error>> {
        let detection_model = Model::load_file(detection_model_path)?;
        let recognition_model = Model::load_file(rec_model_path)?;
        
        Ok(Self {
            detection_model,
            recognition_model,
        })
    }
}
