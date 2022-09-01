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
use crate::service::control_server::{GLOBAL_SOCKET, server_info, server_info_socket};



pub fn init_router() -> Router {
    Router::new()
        .route("/info", get(server_info))
        .route("/ws",get(|socket,arg|server_info_socket(socket,arg,GLOBAL_SOCKET.lock().unwrap().clone())))
}
