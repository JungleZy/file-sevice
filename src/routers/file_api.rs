use axum::{Router};

use crate::api::file_api;

//api
pub fn routers() -> Router {
  Router::new()
  .merge(file_api::init_router())
}