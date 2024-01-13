use std::{cmp::Ordering, path::Path};

use super::{
    base::BaseModel,
    image_processing::{self, process_image},
};
use anyhow::Result;
use image::DynamicImage;
use image_processing::{check_input_image_size, process_ans_image};
use ndarray::Array4;
use rayon::prelude::*;

pub struct M3DRotationPredictor(BaseModel);

impl M3DRotationPredictor {
    /// Create a new instance of the M3DRotationPredictor
    pub fn new(model_dir: Option<&Path>, num_threads: u16) -> Result<Self> {
        Ok(Self(BaseModel::new(
            "3d_rollball_objects_v2.onnx",
            model_dir,
            num_threads,
        )?))
    }

    fn run_prediction(&self, left: &Array4<f32>, right: Array4<f32>) -> Result<Vec<f32>> {
        Ok(self.0.run_prediction(left.to_owned(), right)?)
    }

    /// Predict the rotation of the animal
    ///```
    /// #[inline]
    /// pub fn predict(&self, mut image: &mut DynamicImage) -> Result<i32>> {
    ///     check_input_image_size(&image)?;

    ///     let mut max_prediction = f32::NEG_INFINITY;
    ///     let mut max_index = -1;

    ///     let width = image.width();
    ///     let left = process_ans_image(&mut image, (52, 52))?;
    ///     for i in 0..(width / 200) {
    ///         let right = process_image(&image, (0, i), (52, 52))?;
    ///         let prediction = self.run_prediction(&left, right)?;

    ///         let prediction_value = prediction[0];

    ///         if prediction_value > max_prediction {
    ///             max_prediction = prediction_value;
    ///             max_index = i as i32;
    ///         }
    ///     }
    ///     Ok(max_index)
    /// }
    /// ```

    /// Parallel version of the predict function
    #[inline]
    pub fn predict(&self, mut image: DynamicImage) -> Result<i32> {
        check_input_image_size(&image)?;

        let width = image.width();
        let left = process_ans_image(&mut image, (52, 52))?;

        let results: Result<Vec<(f32, i32)>> = (0..(width / 200))
            .into_par_iter()
            .map(|i| {
                let right = process_image(&image, (0, i), (52, 52))?;
                let prediction = self.run_prediction(&left, right)?;
                let prediction_value = prediction[0];
                Ok((prediction_value, i as i32))
            })
            .collect::<Result<Vec<_>>>();

        let (_, max_index) = results?
            .into_iter()
            .max_by(|(pred1, _), (pred2, _)| pred1.partial_cmp(pred2).unwrap_or(Ordering::Equal))
            .unwrap_or((f32::NEG_INFINITY, -1));

        Ok(max_index)
    }
}
