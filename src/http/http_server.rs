#![allow(unused_variables)] //允许未使用的变量
#![allow(dead_code)] //允许未使用的代码
#![allow(unused_must_use)]

use axum::{
  handler::Handler, 
  http::{Uri}, 
  response::IntoResponse, 
  Router,
  Server
};

use std::{net::SocketAddr};
use std::time::Duration;
use crate::routers::file_api;
use common::RespVO;
use log::warn;
use tower_http::cors::{Any, CorsLayer};

async fn fallback(uri: Uri) -> impl IntoResponse {
  let msg = format!("资源不存在：{}", uri);
  warn!("{}", msg.clone());
  RespVO::<String> {
      code: Some(-1),
      msg: Some(msg),
      data: None,
  }
  .resp_json()
}

#[tokio::main]
pub async fn start(){

  let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any).allow_headers(Any).max_age(Duration::from_secs(60) * 10);
  let app = Router::new()
      .nest("/api/file", file_api::routers())
      .layer(cors)
      .fallback(fallback.into_service());

  let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
  println!("listening on {}", addr);
  Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}