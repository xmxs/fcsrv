use super::{base::ImagePairClassifierPredictor, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct HopscotchHighsecPredictor(ImagePairClassifierPredictor);

impl HopscotchHighsecPredictor {
    /// Create a new instance of the HopscotchHighsecPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(ImagePairClassifierPredictor::new(
            "hopscotch_highsec.onnx",
            args,
        )?))
    }
}

impl Predictor for HopscotchHighsecPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
