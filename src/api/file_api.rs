#![allow(unused_variables)] //允许未使用的变量
#![allow(dead_code)] //允许未使用的代码
#![allow(unused_must_use)]

use axum::{
  response::IntoResponse, 
  routing::{get,post}, 
  Router,
  http::{
    StatusCode, HeaderMap
  }, 
  routing::{get_service},
  extract::{ContentLengthLimit, Multipart},
};
use tower_http::{services::ServeDir};
use common::RespVO;
use rand::prelude::random;
use std::{fs};

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
pub async fn file_info() -> impl IntoResponse {
  let s = "File Service".to_string();
  return RespVO::from(&s).resp_json();
}
pub async fn file_upload(
  ContentLengthLimit(mut multipart): ContentLengthLimit<
      Multipart,{
        1024 * 1024 * 2000 //2000Mb
      },
    >,
    headers: HeaderMap
  ) -> impl IntoResponse {
    if let Some(field) = multipart.next_field().await.unwrap() {
      let name = field.name().unwrap().to_string();
      let file_name = field.file_name().unwrap().to_string();
      let content_type = field.content_type().unwrap().to_string();
      let data = field.bytes().await.unwrap();
      
      let current_path = headers.get("currentPath");
      let mut upload_path = "";
      if current_path.is_none() == false {
        upload_path = current_path.unwrap().to_str().unwrap();
      }
      println!("current_path:{:?}", upload_path);
      let mut save_path =  SAVE_FILE_BASE_PATH.to_string();
      if upload_path != "" {
        save_path = format!("{}/{}", SAVE_FILE_BASE_PATH, upload_path);
      }
      
      println!(
          "Length of `{}` (`{}`: `{}`) is {} bytes",
          name,
          file_name,
          content_type,
          data.len()
      );
      if content_type.starts_with("image/") {
        //根据文件类型生成随机文件名(出于安全考虑)
        let rnd = (random::<f32>() * 1000000000 as f32) as i32;
        //提取"/"的index位置
        let index = content_type
            .find("/")
            .map(|i| i)
            .unwrap_or(usize::max_value());
        //文件扩展名
        let mut ext_name = "xxx";
        if index != usize::max_value() {
            ext_name = &content_type[index + 1..];
        }
        //最终保存在服务器上的文件名
        let save_filename = format!("{}/{}.{}", save_path, rnd, ext_name);

        let fp = fs::read_dir(save_path.to_string());

        if fp.is_err() {
            fs::create_dir(save_path.to_string());
        }
        
        //辅助日志
        println!("filename:{},content_type:{}", save_filename, content_type);
        //保存上传的文件
        tokio::fs::write(&save_filename, &data)
          .await
          .map_err(|err| err.to_string());

        return RespVO::from(&format!("/{}/{}.{}",upload_path, rnd, ext_name)).resp_json();
      }
    }
    let msg = "error".to_string();
    return RespVO::from(&msg).resp_json();
}