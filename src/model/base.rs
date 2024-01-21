use anyhow::Result;
use image::DynamicImage;
use ndarray::Array4;
use ort::{GraphOptimizationLevel, Session};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use sha2::Digest;
use sha2::Sha256;
use std::cmp::Ordering;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::homedir;
use crate::BootArgs;

use super::image_processing::check_input_image_size;
use super::image_processing::process_pair_classifier_ans_image;
use super::image_processing::process_pair_classifier_image;

pub struct BaseModel(Session);

impl BaseModel {
    /// Create a new instance of the BaseModel
    pub fn new(onnx: &'static str, args: &BootArgs) -> Result<Self> {
        let model_dir = args
            .model_dir
            .as_ref()
            .map(|x| x.to_owned())
            .unwrap_or_else(|| {
                homedir::home_dir()
                    .unwrap_or(PathBuf::new())
                    .join(".funcaptcha_models")
            });

        let model_file = Self::initialize_model(onnx, model_dir)?;
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_parallel_execution(false)?
            .with_intra_threads(args.num_threads as i16)?
            .with_allocator(args.allocator)?
            .with_model_from_file(model_file)?;
        Ok(Self(session))
    }
}

/// Download and verify model
impl BaseModel {
    pub fn initialize_model(model_name: &'static str, model_dir: PathBuf) -> Result<String> {
        let model_filename = format!("{}/{model_name}", model_dir.display());

        // Create parent directory if it does not exist
        if let Some(parent_dir) = Path::new(&model_filename).parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(&parent_dir)?;
            }
        }

        let version_url = "https://github.com/MagicalMadoka/funcaptcha-challenger/releases/download/model/version.json";
        let model_url = format!(
            "https://github.com/MagicalMadoka/funcaptcha-challenger/releases/download/model/{model_name}",
        );

        if !Path::new(&model_filename).exists() {
            println!("model {model_name} not found, downloading...");
            Self::download_file(&model_url, &model_filename)?;
        } else {
            println!("model {model_name} found, checking hash");
            let version_json_path = format!("{}/version.json", model_dir.display());

            // Check if version.json exists
            let version_info = if PathBuf::from(&version_json_path).exists() {
                println!("version.json found, checking hash");
                let info: HashMap<String, String> =
                    serde_json::from_str(&fs::read_to_string(&version_json_path)?)?;
                info
            } else {
                println!("version.json not found, downloading...");
                Self::download_file(&version_url, &version_json_path)?;
                let info: HashMap<String, String> =
                    serde_json::from_str(&fs::read_to_string(version_json_path)?)?;
                info
            };

            let expected_hash = &version_info[&model_name
                .split(".")
                .next()
                .ok_or_else(|| anyhow::anyhow!("model name is not valid"))?
                .to_string()];

            println!("expected hash: {}", expected_hash);
            let current_hash = Self::file_sha256(&model_filename)?;
            println!("current hash: {}", current_hash);

            if expected_hash.ne(&current_hash) {
                println!("model {} hash mismatch, downloading...", model_filename);
                Self::download_file(&model_url, &model_filename)?;
            }
        }

        Ok(model_filename)
    }

    fn download_file(url: &str, filename: &str) -> Result<()> {
        let mut response = reqwest::blocking::get(url)?;
        let mut out = fs::File::create(filename)?;
        let mut buffer = [0; 1024];
        println!("downloading {}...", filename);
        while let Ok(n) = response.read(&mut buffer) {
            if n == 0 {
                break;
            }
            out.write_all(&buffer[..n])?;
        }
        drop(out);
        println!("downloaded {} done", filename);
        Ok(())
    }

    fn file_sha256(filename: &str) -> Result<String> {
        let mut file = std::fs::File::open(filename)?;
        let mut sha256 = Sha256::new();
        let mut buffer = [0; 1024];
        while let Ok(n) = file.read(&mut buffer) {
            if n == 0 {
                break;
            }
            sha256.update(&buffer[..n]);
        }
        Ok(format!("{:x}", sha256.finalize()))
    }
}

impl BaseModel {
    /// Run prediction on the model
    pub fn run_prediction(&self, left: Array4<f32>, right: Array4<f32>) -> Result<Vec<f32>> {
        let inputs = ort::inputs! {
            "input_left" => left,
            "input_right" => right,
        }?;

        let outputs = self.0.run(inputs)?;
        let output = outputs[0]
            .extract_tensor::<f32>()?
            .view()
            .t()
            .into_owned()
            .into_iter()
            .map(|x| x)
            .collect();
        return Ok(output);
    }

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
        let left = process_pair_classifier_ans_image(&mut image, (52, 52))?;

        let results: Result<Vec<(f32, i32)>> = (0..(width / 200))
            .into_par_iter()
            .map(|i| {
                let right = process_pair_classifier_image(&image, (0, i), (52, 52))?;
                let prediction = self.run_prediction(left.clone(), right)?;
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
