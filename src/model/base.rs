use anyhow::Result;
use ndarray::Array4;
use ort::{GraphOptimizationLevel, Session};
use sha2::Digest;
use sha2::Sha256;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::homedir;

pub struct BaseModel(Session);

impl BaseModel {
    /// Create a new instance of the BaseModel
    pub fn new(onnx: &'static str, model_dir: Option<&Path>, num_threads: u16) -> Result<Self> {
        let model_dir = model_dir.map(|x| x.to_owned()).unwrap_or_else(|| {
            homedir::home_dir()
                .unwrap_or(PathBuf::new())
                .join(".funcaptcha_models")
        });

        let model_file = Self::initialize_model(onnx, model_dir)?;
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_parallel_execution(false)?
            .with_intra_threads(num_threads as i16)?
            .with_model_from_file(model_file)?;
        Ok(Self(session))
    }

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
}

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
