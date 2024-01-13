use std::{path::Path, str::FromStr};

use self::m3d_rollball_objects::M3DRotationPredictor;
use anyhow::Result;
use serde::Deserialize;
use tokio::sync::OnceCell;

mod base;
mod image_processing;
mod m3d_rollball_objects;

pub static M3D_ROLLBALL_PREDICTOR: OnceCell<M3DRotationPredictor> = OnceCell::const_new();

/// Load the models
pub fn load_models(model_dir: Option<&Path>, num_threads: u16) -> Result<()> {
    let m3d_rotation_predictor = M3DRotationPredictor::new(model_dir, num_threads)?;
    M3D_ROLLBALL_PREDICTOR
        .set(m3d_rotation_predictor)
        .map_err(|_| anyhow::anyhow!("failed to load models"))?;
    Ok(())
}

/// Get the model predictor for the given model type
pub fn get_model_predictor(model_type: ModelType) -> Result<&'static M3DRotationPredictor> {
    match model_type {
        ModelType::M3dRollballAnimals | ModelType::M3dRollballObjects => M3D_ROLLBALL_PREDICTOR
            .get()
            .ok_or_else(|| anyhow::anyhow!("models not loaded")),
    }
}

#[derive(Debug)]
pub enum ModelType {
    M3dRollballAnimals,
    M3dRollballObjects,
}

impl<'de> Deserialize<'de> for ModelType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "3d_rollball_animals" => Ok(ModelType::M3dRollballAnimals),
            "3d_rollball_objects" => Ok(ModelType::M3dRollballObjects),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["3d_rollball_animals", "3d_rollball_objects"],
            )),
        }
    }
}

impl FromStr for ModelType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "3d_rollball_animals" => Ok(Self::M3dRollballAnimals),
            "3d_rollball_objects" => Ok(Self::M3dRollballObjects),
            _ => Err(anyhow::anyhow!("invalid model type")),
        }
    }
}
