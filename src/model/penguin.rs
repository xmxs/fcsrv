use super::{base::ImageClassifierPredictor, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct PenguinPredictor(ImageClassifierPredictor);

impl PenguinPredictor {
    /// Create a new instance of the TrainCoordinatesPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(ImageClassifierPredictor::new("penguin.onnx", args)?))
    }
}

impl Predictor for PenguinPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
