mod base;
mod coordinatesmatch;
mod hopscotch_highsec;
mod image_processing;
mod m3d_rollball_objects;
mod penguin;
mod shadows;
mod train_coordinates;

use self::{
    coordinatesmatch::CoordinatesMatchPredictor, hopscotch_highsec::HopscotchHighsecPredictor,
    m3d_rollball_objects::M3DRotationPredictor, penguin::PenguinPredictor,
    shadows::ShadowsPredictor, train_coordinates::TrainCoordinatesPredictor,
};
use crate::BootArgs;
use anyhow::Result;
use image::DynamicImage;
use serde::Deserialize;
use tokio::sync::OnceCell;

static M3D_ROLLBALL_PREDICTOR: OnceCell<M3DRotationPredictor> = OnceCell::const_new();
static COORDINATES_MATCH_PREDICTOR: OnceCell<CoordinatesMatchPredictor> = OnceCell::const_new();
static HOPSCOTCH_HIGHSEC_PREDICTOR: OnceCell<HopscotchHighsecPredictor> = OnceCell::const_new();
static TRAIN_COORDINATES_PREDICTOR: OnceCell<TrainCoordinatesPredictor> = OnceCell::const_new();
static PENGUIN_PREDICTOR: OnceCell<PenguinPredictor> = OnceCell::const_new();
static SHADOWS_PREDICTOR: OnceCell<ShadowsPredictor> = OnceCell::const_new();

/// Predictor trait
pub trait Predictor: Send + Sync {
    fn predict(&self, image: DynamicImage) -> Result<i32>;
}

/// Load the models predictor
pub fn init_predictor(args: &BootArgs) -> Result<()> {
    set_predictor(&M3D_ROLLBALL_PREDICTOR, || M3DRotationPredictor::new(args))?;
    set_predictor(&COORDINATES_MATCH_PREDICTOR, || {
        CoordinatesMatchPredictor::new(args)
    })?;
    set_predictor(&HOPSCOTCH_HIGHSEC_PREDICTOR, || {
        HopscotchHighsecPredictor::new(args)
    })?;
    set_predictor(&TRAIN_COORDINATES_PREDICTOR, || {
        TrainCoordinatesPredictor::new(args)
    })?;
    set_predictor(&PENGUIN_PREDICTOR, || PenguinPredictor::new(args))?;
    set_predictor(&SHADOWS_PREDICTOR, || ShadowsPredictor::new(args))?;
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
        ModelType::TrainCoordinates => get_predictor_from_cell(&TRAIN_COORDINATES_PREDICTOR)?,
        ModelType::Penguin => get_predictor_from_cell(&PENGUIN_PREDICTOR)?,
        ModelType::Shadows => get_predictor_from_cell(&SHADOWS_PREDICTOR)?,
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
    TrainCoordinates,
    Penguin,
    Shadows,
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
            "train_coordinates" => Ok(ModelType::TrainCoordinates),
            "penguin" => Ok(ModelType::Penguin),
            "shadows" => Ok(ModelType::Shadows),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "3d_rollball_animals",
                    "3d_rollball_objects",
                    "coordinatesmatch",
                    "hopscotch_highsec",
                    "train_coordinates",
                    "penguin",
                    "shadows",
                ],
            )),
        }
    }
}
