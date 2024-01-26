mod task;

use std::convert::Infallible;

use self::task::{Task, TaskResult};
use crate::{model, BootArgs};
use anyhow::Result;
use image::DynamicImage;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use reqwest::StatusCode;
use tokio::sync::OnceCell;
use warp::filters::body::BodyDeserializeError;
use warp::reject::{Reject, Rejection};
use warp::reply::Reply;
use warp::Filter;

static API_KEY: OnceCell<Option<String>> = OnceCell::const_new();
static SUBMIT_LIMIT: OnceCell<Option<usize>> = OnceCell::const_new();

pub struct Serve(BootArgs);

impl Serve {
    pub fn new(args: BootArgs) -> Self {
        Self(args)
    }

    #[tokio::main]
    pub async fn run(self) -> Result<()> {
        // Init API key
        API_KEY.set(self.0.api_key)?;

        // Init submit limit
        SUBMIT_LIMIT.set(Some(self.0.multi_image_limit))?;

        // Init routes
        let routes = warp::path("task")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handle_task)
            .recover(handle_rejection)
            .with(warp::trace::request());

        tracing::info!("Listening on {}", self.0.bind);

        // Start the server
        match (self.0.tls_cert, self.0.tls_key) {
            (Some(cert), Some(key)) => {
                warp::serve(routes)
                    .tls()
                    .cert_path(cert)
                    .key_path(key)
                    .bind_with_graceful_shutdown(self.0.bind, async {
                        tokio::signal::ctrl_c()
                            .await
                            .expect("failed to install CTRL+C signal handler");
                    })
                    .1
                    .await;
            }
            _ => {
                warp::serve(routes)
                    .bind_with_graceful_shutdown(self.0.bind, async {
                        tokio::signal::ctrl_c()
                            .await
                            .expect("failed to install CTRL+C signal handler");
                    })
                    .1
                    .await;
            }
        }
        Ok(())
    }
}

/// Handle the task
async fn handle_task(task: Task) -> Result<impl Reply, Rejection> {
    // Check the API key
    check_api_key(task.api_key).await?;
    // Check the submit limit
    check_submit_limit(task.images.len()).await?;

    // Solve the task
    match model::get_predictor(task.typed) {
        Ok(predictor) => {
            let objects = if task.images.len() == 1 {
                let image = decode_image(&task.images[0])
                    .map_err(|e| warp::reject::custom(BadRequest(e.to_string())))?;
                let answer = predictor
                    .predict(image)
                    .map_err(|e| warp::reject::custom(BadRequest(e.to_string())))?;

                vec![answer as u32]
            } else {
                let mut objects = task
                    .images
                    .into_par_iter()
                    .enumerate()
                    .map(|(index, image)| {
                        // decode the image
                        let image = decode_image(&image)?;
                        let answer = predictor.predict(image)?;
                        Ok((index, answer as u32))
                    })
                    .collect::<Result<Vec<(usize, u32)>>>()
                    .map_err(|e| warp::reject::custom(BadRequest(e.to_string())))?;

                objects.sort_by_key(|&(index, _)| index);
                objects
                    .into_iter()
                    .map(|(_, answer)| answer)
                    .collect::<Vec<u32>>()
            };

            let result = TaskResult {
                error: None,
                solve: true,
                objects,
            };
            return Ok(warp::reply::json(&result));
        }
        Err(e) => {
            return Err(warp::reject::custom(BadRequest(e.to_string())));
        }
    }
}

/// Check the API key
async fn check_api_key(api_key: Option<String>) -> Result<(), Rejection> {
    if let Some(Some(key)) = API_KEY.get() {
        if let Some(api_key) = api_key {
            if key.ne(&api_key) {
                return Err(warp::reject::custom(InvalidTApiKeyError));
            }
        } else {
            return Err(warp::reject::custom(InvalidTApiKeyError));
        }
    }
    Ok(())
}

/// Check the submit limit
async fn check_submit_limit(len: usize) -> Result<(), Rejection> {
    if let Some(Some(limit)) = SUBMIT_LIMIT.get() {
        if len > *limit {
            return Err(warp::reject::custom(InvalidSubmitLimitError));
        }
    }
    Ok(())
}

/// Decode the base64 image
fn decode_image(base64_string: &String) -> Result<DynamicImage> {
    // base64 decode the image
    use base64::{engine::general_purpose, Engine as _};
    let image_bytes = general_purpose::STANDARD
        .decode(base64_string.split(',').nth(1).unwrap_or(base64_string))?;
    // convert the bytes to an image
    Ok(image::load_from_memory(&image_bytes)?)
}

#[derive(Debug)]
struct BadRequest(String);

#[derive(Debug)]
struct InvalidTApiKeyError;

#[derive(Debug)]
struct InvalidSubmitLimitError;

impl Reject for BadRequest {}

impl Reject for InvalidTApiKeyError {}

impl Reject for InvalidSubmitLimitError {}

impl Reject for TaskResult {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found".to_owned();
    } else if let Some(e) = err.find::<BadRequest>() {
        code = StatusCode::BAD_REQUEST;
        message = e.0.to_owned();
    } else if let Some(_) = err.find::<InvalidTApiKeyError>() {
        code = StatusCode::UNAUTHORIZED;
        message = "Invalid API key".to_owned();
    } else if let Some(e) = err.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = e.to_string();
    } else if let Some(_) = err.find::<InvalidSubmitLimitError>() {
        code = StatusCode::BAD_REQUEST;
        if let Some(limit) = SUBMIT_LIMIT.get() {
            message = format!("Invalid submit limit: {}", limit.unwrap_or(0));
        } else {
            message = "Invalid submit limit".to_owned();
        }
    } else {
        tracing::info!("Unhandled application error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error".to_owned();
    }

    let json = warp::reply::json(&TaskResult {
        error: Some(message),
        solve: false,
        objects: vec![],
    });

    Ok(warp::reply::with_status(json, code))
}
