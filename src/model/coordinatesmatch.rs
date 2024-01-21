use super::{base::ImagePairClassifierPredictor, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct CoordinatesMatchPredictor(ImagePairClassifierPredictor);

impl CoordinatesMatchPredictor {
    /// Create a new instance of the CoordinatesMatchPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(ImagePairClassifierPredictor::new(
            "coordinatesmatch.onnx",
            args,
        )?))
    }
}

impl Predictor for CoordinatesMatchPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
