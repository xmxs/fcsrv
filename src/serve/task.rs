use crate::model::ModelType;
use anyhow::Error as AnyhowError;
use image::ImageError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Task {
    /// API key
    pub api_key: Option<String>,
    /// model type, e.g. 3d_rollball_animals
    #[serde(rename = "type")]
    pub typed: ModelType,
    /// base64 image list, e.g. ["/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkS"]
    pub images: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskResult {
    /// error message, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// whether the model is a solve
    pub solve: bool,
    /// whether the model is a classifier
    pub objects: Vec<u32>,
}

impl From<ImageError> for TaskResult {
    fn from(result: ImageError) -> Self {
        Self {
            error: Some(result.to_string()),
            solve: false,
            objects: vec![],
        }
    }
}
impl From<base64::DecodeError> for TaskResult {
    fn from(result: base64::DecodeError) -> Self {
        Self {
            error: Some(result.to_string()),
            solve: false,
            objects: vec![],
        }
    }
}

impl From<AnyhowError> for TaskResult {
    fn from(err: AnyhowError) -> Self {
        Self {
            error: Some(err.to_string()),
            solve: false,
            objects: vec![],
        }
    }
}
