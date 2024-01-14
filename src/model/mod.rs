mod base;
mod coordinatesmatch;
mod hopscotch_highsec;
mod image_processing;
mod m3d_rollball_objects;

use self::{
    coordinatesmatch::CoordinatesMatchPredictor, hopscotch_highsec::HopscotchHighsecPredictor,
    m3d_rollball_objects::M3DRotationPredictor,
};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;
use serde::Deserialize;
use tokio::sync::OnceCell;

static M3D_ROLLBALL_PREDICTOR: OnceCell<M3DRotationPredictor> = OnceCell::const_new();
static COORDINATES_MATCH_PREDICTOR: OnceCell<CoordinatesMatchPredictor> = OnceCell::const_new();
static HOPSCOTCH_HIGHSEC_PREDICTOR: OnceCell<HopscotchHighsecPredictor> = OnceCell::const_new();

/// Predictor trait
pub trait Predictor {
    fn predict(&self, image: DynamicImage) -> Result<i32>;
}

/// Load the models predictor
pub fn load_predictor(args: &BootArgs) -> Result<()> {
    set_predictor(&M3D_ROLLBALL_PREDICTOR, || M3DRotationPredictor::new(args))?;
    set_predictor(&COORDINATES_MATCH_PREDICTOR, || {
        CoordinatesMatchPredictor::new(args)
    })?;
    set_predictor(&HOPSCOTCH_HIGHSEC_PREDICTOR, || {
        HopscotchHighsecPredictor::new(args)
    })?;
    Ok(())
}

/// Get the model predictor for the given model type
pub fn get_predictor(model_type: ModelType) -> Result<&'static dyn Predictor> {
    let predictor = match model_type {
        ModelType::M3dRollballAnimals | ModelType::M3dRollballObjects => {
            get_predictor_from_cell(&M3D_ROLLBALL_PREDICTOR)?
        }
        ModelType::Coordinatesmatch => get_predictor_from_cell(&COORDINATES_MATCH_PREDICTOR)?,
        ModelType::HopscotchHighsec => get_predictor_from_cell(&HOPSCOTCH_HIGHSEC_PREDICTOR)?,
    };
    Ok(predictor)
}

fn set_predictor<P, F>(cell: &OnceCell<P>, creator: F) -> Result<()>
where
    P: Predictor,
    F: FnOnce() -> Result<P>,
{
    cell.set(creator()?)
        .map_err(|_| anyhow::anyhow!("failed to load models"))
}

fn get_predictor_from_cell<P>(cell: &'static OnceCell<P>) -> Result<&'static dyn Predictor>
where
    P: Predictor + 'static,
{
    let predictor = cell
        .get()
        .ok_or_else(|| anyhow::anyhow!("models not loaded"))?;
    Ok(predictor as &'static dyn Predictor)
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
