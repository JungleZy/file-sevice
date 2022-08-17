#![allow(unused_variables)] //允许未使用的变量
#![allow(dead_code)] //允许未使用的代码
#![allow(unused_must_use)]

use axum::{
  routing::{get,post}, 
  Router,
  http::{
    StatusCode
  }, 
  routing::{get_service},
};
use tower_http::{services::ServeDir};
use crate::service::file_service::{
  file_info,
  file_upload,
};

const SAVE_FILE_BASE_PATH: &str = "./file";

pub fn init_router() -> Router {
  Router::new()
    .route("/info", get(file_info))
    .route("/upload", post(file_upload))
    .nest("/getFile", get_service(ServeDir::new(SAVE_FILE_BASE_PATH))
    .handle_error(|error: std::io::Error| async move {(
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Unhandled internal error: {}", error),
    )}))
}
