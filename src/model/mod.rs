mod base;
mod coordinatesmatch;
mod hopscotch_highsec;
mod image_processing;
mod m3d_rollball_objects;

use self::m3d_rollball_objects::M3DRotationPredictor;
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;
use serde::Deserialize;
use std::str::FromStr;
use tokio::sync::OnceCell;

static M3D_ROLLBALL_PREDICTOR: OnceCell<M3DRotationPredictor> = OnceCell::const_new();
static COORDINATES_MATCH_PREDICTOR: OnceCell<coordinatesmatch::CoordinatesMatchPredictor> =
    OnceCell::const_new();
static HOPSCOTCH_HIGHSEC_PREDICTOR: OnceCell<hopscotch_highsec::HopscotchHighsecPredictor> =
    OnceCell::const_new();

/// Predictor trait
pub trait Predictor {
    fn predict(&self, image: DynamicImage) -> Result<i32>;
}

/// Load the models
pub fn load_models(args: &BootArgs) -> Result<()> {
    let m3d_rotation_predictor = M3DRotationPredictor::new(args)?;
    M3D_ROLLBALL_PREDICTOR
        .set(m3d_rotation_predictor)
        .map_err(|_| anyhow::anyhow!("failed to load models"))?;
    COORDINATES_MATCH_PREDICTOR
        .set(coordinatesmatch::CoordinatesMatchPredictor::new(args)?)
        .map_err(|_| anyhow::anyhow!("failed to load models"))?;
    HOPSCOTCH_HIGHSEC_PREDICTOR
        .set(hopscotch_highsec::HopscotchHighsecPredictor::new(args)?)
        .map_err(|_| anyhow::anyhow!("failed to load models"))?;
    Ok(())
}

/// Get the model predictor for the given model type
pub fn get_predictor(model_type: ModelType) -> Result<&'static dyn Predictor> {
    let predictor: &'static dyn Predictor = match model_type {
        ModelType::M3dRollballAnimals | ModelType::M3dRollballObjects => M3D_ROLLBALL_PREDICTOR
            .get()
            .ok_or_else(|| anyhow::anyhow!("models not loaded"))?
            as &'static dyn Predictor,
        ModelType::Coordinatesmatch => COORDINATES_MATCH_PREDICTOR
            .get()
            .ok_or_else(|| anyhow::anyhow!("models not loaded"))?
            as &'static dyn Predictor,
        ModelType::HopscotchHighsec => HOPSCOTCH_HIGHSEC_PREDICTOR
            .get()
            .ok_or_else(|| anyhow::anyhow!("models not loaded"))?
            as &'static dyn Predictor,
    };
    Ok(predictor)
}

#[derive(Debug)]
pub enum ModelType {
    M3dRollballAnimals,
    M3dRollballObjects,
    Coordinatesmatch,
    HopscotchHighsec,
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
            "coordinatesmatch" => Ok(ModelType::Coordinatesmatch),
            "hopscotch_highsec" => Ok(ModelType::HopscotchHighsec),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "3d_rollball_animals",
                    "3d_rollball_objects",
                    "coordinatesmatch",
                    "hopscotch_highsec",
                ],
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
