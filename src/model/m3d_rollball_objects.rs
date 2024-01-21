use super::{base::ImagePairClassifierPredictor, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct M3DRotationPredictor(ImagePairClassifierPredictor);

impl M3DRotationPredictor {
    /// Create a new instance of the M3DRotationPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(ImagePairClassifierPredictor::new(
            "3d_rollball_objects_v2.onnx",
            args,
        )?))
    }
}

impl Predictor for M3DRotationPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
