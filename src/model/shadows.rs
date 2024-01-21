use super::{base::ImageClassifierPredictor, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct ShadowsPredictor(ImageClassifierPredictor);

impl ShadowsPredictor {
    /// Create a new instance of the TrainCoordinatesPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(ImageClassifierPredictor::new("shadows.onnx", args)?))
    }
}

impl Predictor for ShadowsPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
