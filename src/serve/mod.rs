mod task;

use std::convert::Infallible;

use self::task::{Task, TaskResult};
use crate::{model, BootArgs};
use anyhow::Result;
use image::DynamicImage;
use reqwest::StatusCode;
use tokio::sync::OnceCell;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use warp::filters::body::BodyDeserializeError;
use warp::reject::{Reject, Rejection};
use warp::reply::Reply;

static API_KEY: OnceCell<Option<String>> = OnceCell::const_new();

pub struct Serve(BootArgs);

impl Serve {
    pub fn new(args: BootArgs) -> Self {
        Self(args)
    }

    #[tokio::main]
    pub async fn run(self) -> Result<()> {
        if self.0.debug {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
        // Init tracing
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "RUST_LOG=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        // Init API key
        API_KEY.set(self.0.api_key)?;

        // Init routes
        use warp::filters::BoxedFilter;
        use warp::Filter;

        fn with_auth() -> BoxedFilter<(Result<String, ()>,)> {
            warp::header("authorization")
                .map(|token: String| Ok(token))
                .boxed()
        }

        fn default_auth() -> BoxedFilter<(Result<String, ()>,)> {
            warp::path("task")
                .and(warp::post())
                .and(warp::any().map(|| Err(())))
                .boxed()
        }

        let routes = with_auth()
            .or(default_auth())
            .unify()
            .and(warp::body::json())
            .and_then(handle_task)
            .recover(handle_rejection)
            .with(warp::trace::request());

        // Start the server
        match (self.0.tls_cert, self.0.tls_key) {
            (Some(cert), Some(key)) => {
                warp::serve(routes)
                    .tls()
                    .cert_path(cert)
                    .key_path(key)
                    .run(self.0.bind)
                    .await;
            }
            _ => {
                warp::serve(routes).run(self.0.bind).await;
            }
        }
        Ok(())
    }
}

/// Handle the task
async fn handle_task(auth: Result<String, ()>, task: Task) -> Result<impl Reply, Rejection> {
    // Check the API key
    check_api_key(auth).await?;

    // Solve the task
    match model::get_predictor(task.typed) {
        Ok(predictor) => {
            // decode the image
            let image = decode_image(task.image)
                .map_err(|e| warp::reject::custom(BadRequest(e.to_string())))?;

            let objects = predictor
                .predict(image)
                .map_err(|e| warp::reject::custom(BadRequest(e.to_string())))?;

            let result = TaskResult {
                error: None,
                solve: true,
                objects: vec![objects as u32],
            };
            return Ok(warp::reply::json(&result));
        }
        Err(e) => {
            return Err(warp::reject::custom(BadRequest(e.to_string())));
        }
    }
}

/// Check the API key
async fn check_api_key(auth: Result<String, ()>) -> Result<(), Rejection> {
    if let Some(Some(api_key)) = API_KEY.get() {
        if let Ok(token) = auth {
            if !token.starts_with("Bearer ") || api_key.ne(&token["Bearer ".len()..]) {
                return Err(warp::reject::custom(InvalidTokenError));
            }
        } else {
            return Err(warp::reject::custom(InvalidTokenError));
        }
    }
    Ok(())
}

/// Decode the base64 image
fn decode_image(base64_string: String) -> Result<DynamicImage> {
    // base64 decode the image
    use base64::{engine::general_purpose, Engine as _};
    let image_bytes = general_purpose::STANDARD
        .decode(base64_string.split(',').nth(1).unwrap_or(&base64_string))?;
    // convert the bytes to an image
    Ok(image::load_from_memory(&image_bytes)?)
}

#[derive(Debug)]
struct BadRequest(String);

#[derive(Debug)]
struct InvalidTokenError;

impl Reject for BadRequest {}

impl Reject for InvalidTokenError {}

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
    } else if let Some(_) = err.find::<InvalidTokenError>() {
        code = StatusCode::UNAUTHORIZED;
        message = "Invalid Token".to_owned();
    } else if let Some(e) = err.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = e.to_string();
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
