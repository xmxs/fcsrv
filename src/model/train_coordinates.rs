use super::{base::BaseModel, Predictor};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;

pub struct TrainCoordinatesPredictor(BaseModel);

impl TrainCoordinatesPredictor {
    /// Create a new instance of the TrainCoordinatesPredictor
    pub fn new(args: &BootArgs) -> Result<Self> {
        Ok(Self(BaseModel::new("train_coordinates.onnx", args)?))
    }
}

impl Predictor for TrainCoordinatesPredictor {
    fn predict(&self, image: DynamicImage) -> Result<i32> {
        self.0.predict(image)
    }
}
