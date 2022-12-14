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
    remove_dir_or_file,
    down_load_file,
    compressed_file,
    uncompressed_file,
    GLOBAL_SOCKET,
    websocket_handle,
    create_dir,
};

const SAVE_FILE_BASE_PATH: &str = "./file";

pub fn init_router() -> Router {
  Router::new()
      .route("/info", get(file_info))
      .route("/upload", post(file_upload))
      .route("/create",post(create_dir))
      .route("/remove", post(remove_dir_or_file))
      .route("/downFile",get(down_load_file))
      .route("/compressedFile",post(compressed_file))//压缩
      .route("/uncompressedFile",get(uncompressed_file))//解压
      //websocket推送压缩、解压进度
      .route("/ws",get(|socket,args|websocket_handle(socket,GLOBAL_SOCKET.lock().unwrap().clone(),args)))
      //静态资源显示
      .nest("/getFile", get_service(ServeDir::new(SAVE_FILE_BASE_PATH))
          .handle_error(|error: std::io::Error| async move {
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              format!("Unhandled internal error: {}", error),
            )
          }))
}
