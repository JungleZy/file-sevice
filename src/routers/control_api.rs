use axum::{Router};

use crate::api::control_api;

//监控api
pub fn routers() -> Router {
    Router::new()
        .merge(control_api::init_router())
}