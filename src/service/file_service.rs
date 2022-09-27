#![allow(unused_variables)] //允许未使用的变量
#![allow(dead_code)] //允许未使用的代码
#![allow(unused_must_use)]
#[warn(unused_mut)]
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{copy, Read, Seek, Write};
use std::path::{Path as SPath};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use std::vec::IntoIter;
use axum::{response::IntoResponse, extract::{ContentLengthLimit, Multipart}, http::HeaderMap, Json};
use axum::body::Body;
use axum::extract::{Query, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::Response;
use common::RespVO;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::broadcast::Receiver;
use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;
use crate::entity::file_entity::{FileAppState, FileEntity, FileSocketData};
use crate::entity::query::file_query::{CompressedFilesParam, RemoveFileQuery, UnCompressedFileParam};

const SAVE_FILE_BASE_PATH: &str = ".\\file";
//服务管理目录
const SERVER_MANAGE_SAVE_FILE_BASE_PATH:&str = ".\\serverManage";

//初始化全局socket线程管理器
pub static GLOBAL_SOCKET:Lazy<Mutex<Arc<FileAppState>>> = Lazy::new(||{
  let app_state = FileAppState::new();
  Mutex::new(app_state)
});

//获取文件列表
pub async fn file_info(query:Query<HashMap<String,String>>) -> impl IntoResponse {
  let option = query.0.get("path");
  let is_server_manage = query.0.get("isServerManage");
  let mut show_path = SAVE_FILE_BASE_PATH.to_string();
  if is_server_manage.is_some() {
    let flag = is_server_manage.unwrap();
    if flag.eq("true") {
      show_path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
    }
  }
  if let Some(path) = option{
    show_path.push_str("//");
    show_path.push_str(path);
  }
  let ret:Vec<FileEntity> = read_current_dir(&show_path);

  return RespVO::from(&ret).resp_json();
}

//读取当前目录
fn read_current_dir(path:&str)->Vec<FileEntity>{
  let dir = fs::read_dir(path);
  let mut  ret:Vec<FileEntity> = Vec::new();
  if let Ok(dir) = dir{
    for file in dir {
      if let Ok(file) = file{
        let if_file = file.file_type().unwrap().is_file();
        let mut file_size = 0;
        let mut update_date = 0;
        let mut file_type = 0;
        if if_file {
          file_size = file.metadata().unwrap().len();
          update_date = file.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis();
          file_type = 1;
        }
        let  file_name = file.file_name().to_str().unwrap().to_string();
        let file_entity = FileEntity::new(file_name, file_size, update_date, file_type);
        ret.push(file_entity);
      }
    }
  }

  ret
}

//文件上传
pub async fn file_upload(
  ContentLengthLimit(mut multipart): ContentLengthLimit<
      Multipart,{
        1024 * 1024 * 2000 //2000Mb
      },
    >,
    headers: HeaderMap
  ) -> impl IntoResponse {
    let mut msg = "error".to_string();

    if let Some(field) = multipart.next_field().await.unwrap() {
      let name = field.name().unwrap().to_string();
      let file_name = field.file_name().unwrap().to_string();
      println!("原始文件名称：{}",file_name);
      let content_type = field.content_type().unwrap().to_string();
      let data = field.bytes().await.unwrap();
      
      let current_path = headers.get("currentPath");
      let is_server_manage = headers.get("isServerManage");

      let mut upload_path = "";
      if current_path.is_none() == false {
        upload_path = current_path.unwrap().to_str().unwrap();
      }
      let mut save_path =  SAVE_FILE_BASE_PATH.to_string();
      if is_server_manage.is_none() == false {
        save_path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
      }
      if upload_path != "" {
        // save_path = format!("{}/{}", SAVE_FILE_BASE_PATH, upload_path);
        save_path.push_str("\\");
        save_path.push_str(upload_path);
      }
      // fs::create_dir_all(save_path.to_string()).unwrap();
      println!(
          "Length of `{}` (`{}`: `{}`) is {} bytes",
          name,
          file_name,
          content_type,
          data.len()
      );

        //最终保存在服务器上的文件名
        let save_filename = format!("{}/{}", save_path, file_name);

        let fp = fs::read_dir(save_path.to_string());

        if fp.is_err() {
            fs::create_dir_all(save_path.to_string()).unwrap();
        }
        
        //辅助日志
        println!("filename:{},content_type:{}", save_filename, content_type);
        //保存上传的文件
        tokio::fs::write(&save_filename, &data)
            .await
            .map_err(|err| msg = err.to_string());

        return RespVO::from(&format!("/{}/{}",upload_path, file_name)).resp_json();
      // }
    }
    
    return RespVO::from(&msg).resp_json();
}


//删除文件夹或文件 path:&str,is_file:bool
pub async fn remove_dir_or_file(Json(query):Json<RemoveFileQuery>) -> impl IntoResponse{
  let mut full_path = SAVE_FILE_BASE_PATH.to_string();
  if query.is_server_manage {
      full_path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
  }
  full_path.push_str("/");
  full_path.push_str(query.path.as_str());

  let is_file = query.is_file;
  let ok:String = "ok".to_string();
  //删除文件
  if is_file{
    let result = fs::remove_file(full_path);
    if let Err(err) = result{
      let msg = err.to_string();
      return RespVO::from_error(msg,String::from("")).resp_json();
    }
  }
  //     删除文件夹
  else {
    let result = fs::remove_dir_all(full_path);
    if let Err(e) = result{
      let msg = e.to_string();
      return RespVO::from_error(msg,String::from("")).resp_json();
    }
  }
  return  RespVO::from(&ok).resp_json();
}


//下载文件 fileName = "下载路径" isServerManage = "是否是服务管理目录"
pub async fn down_load_file(file_name:Query<HashMap<String,String>>) -> impl IntoResponse {
  let name = file_name.0.get("fileName");
  let is_server_manage = file_name.0.get("isServerManage");
  if let None = name{
    return  RespVO::from_error(String::from("请传入下载文件路径名称"),String::from(" ")).resp_json();
  }
  let name = name.unwrap();
  let mut file_name = String::from("attachment;filename=");
  if name.contains("/"){
    let v:Vec<&str> = name.rsplit("/").collect();
    let x = v.get(0).unwrap();
    file_name.push_str(x);
  }else {
    file_name.push_str(name);
  }
  let mut file_path = String::from(SAVE_FILE_BASE_PATH.clone());
  if is_server_manage.is_some() == true {
    file_path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
  }
  file_path.push_str("/");
  file_path.push_str(name);
  let result = fs::read(file_path);
  if let Err(msg) = result{
    return RespVO::from_error(msg.to_string(),String::from(" ")).resp_json();
  }
  let data = result.ok().unwrap();


  Response::builder()
      .extension(||{})
      .header("Content-Disposition",file_name)
      .header("Content-Type","application/octet-stream")
      .body(Body::from(data))
      .unwrap()

}


//压缩文件
pub async fn compressed_file(Json(param):Json<CompressedFilesParam>)-> impl IntoResponse {
  //异步压缩
  tokio::spawn(async move{
    let mut path = SAVE_FILE_BASE_PATH.to_string();
    if param.is_server_manage {
      path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
    }
    path.push_str("\\");
    path.push_str(param.path.as_str());

    let mut target_path = path.clone();
    target_path.push_str(param.zip_name.as_str());
    target_path.push_str(".zip");

    //对要进行压缩的文件进行拼接
    let mut new_files:Vec<String> = vec![];
    for file in param.files {
      let mut temp_path = path.clone();
      temp_path.push_str(file.as_str());
      new_files.push(temp_path);
    }
    compress_dir(SPath::new(path.as_str()), SPath::new(target_path.as_str()), new_files);
  });
  let ok = String::from("正在压缩");
  return RespVO::from(&ok).resp_json();
}


//解压
pub async fn uncompressed_file(Json(param):Json<UnCompressedFileParam>)->impl IntoResponse{
  let ok = String::from("正在解压");
  //异步解压文件
  tokio::spawn(async move{
    let mut zip_path = SAVE_FILE_BASE_PATH.to_string();
    if param.is_server_manage {
      zip_path = SERVER_MANAGE_SAVE_FILE_BASE_PATH.to_string();
    }
    zip_path.push_str("\\");
    zip_path.push_str(param.path.as_str());
    zip_path.push_str("\\");
    zip_path.push_str(param.zip_name.as_str());
    let mut compressed_path = SAVE_FILE_BASE_PATH.to_string();
    compressed_path.push_str("\\");
    compressed_path.push_str(param.path.as_str());
    extract(SPath::new(zip_path.as_str()),SPath::new(compressed_path.as_str()));
  });

  return RespVO::from(&ok).resp_json();

}

/// 压缩文件夹
/// test文件夹下有a.jpg和b.txt 两个文件
/// 压缩成test.zip文件
fn compress_dir(src_dir: &SPath, target: &SPath, files:Vec<String>) {
  if let Err(e) = std::fs::File::create(target){
    println!("{}",e.to_string());
  }
  let zipfile = std::fs::File::create(target).unwrap();
  let dir = WalkDir::new(src_dir);
  let dir = dir.sort_by_file_name();

  let compress_path = target.clone().parent().unwrap().to_str().unwrap();
  let file_name = target.clone().file_name().unwrap().to_str().unwrap().to_string();

  //过滤掉其它文件
  let dir_entry:Vec<_> = dir.into_iter().filter_map(|i| i.ok()).collect();
  let mut new_dir_entry:Vec<_> = vec![];
  for entry in dir_entry {
    let path = entry.path().to_str().unwrap();
    for file in &files {
      if path.starts_with(file) {
        new_dir_entry.push(entry);
        break;
      }
    }
  }
  if new_dir_entry.len() == 0 {

  }

  let result = zip_dir(new_dir_entry.into_iter(),src_dir.to_str().unwrap(),zipfile,compress_path,file_name);

  if let Err(e) = result{
    println!("{}",e.to_string());
    //return RespVO::from_error(e.to_string(),String::from("")).resp_json();
  }
}

fn zip_dir<T>(it: IntoIter<DirEntry>, prefix: &str, writer: T, compress_path: &str,file_name:String) -> zip::result::ZipResult<()>
  where T: Write + Seek {
  let mut zip = zip::ZipWriter::new(writer);
  let options = FileOptions::default()
      .compression_method(zip::CompressionMethod::Bzip2)//直接用了bzip2压缩方式，其它参看枚举
      .unix_permissions(0o755);//unix系统权限

  let mut buffer = Vec::new();
  //偏移量
  let mut index = 1;
  let total = it.len();

  if total == 0{
    send_to_socket(
      1,
      0,
      total as u64,
      compress_path.to_string(),
      1,
      file_name.clone());
    println!("没有可压缩的文件");
    return Result::Err(ZipError::FileNotFound);
  }

  for entry in it {
    let path = entry.path();
    //zip压缩一个文件时，会把它的全路径当成文件名(在下面的解压函数中打印文件名可知)
    //这里是去掉目录前缀
    let name = path.strip_prefix(SPath::new(prefix)).unwrap();

    // Write file or directory explicitly
    // Some unzip tools unzip files with directory paths correctly, some do not!
    if path.is_file() {
      #[allow(deprecated)]
      let start_file = zip.start_file_from_path(name, options);
      if let Err(e) = start_file{
        //发送失败消息到socket中
        send_to_socket(
          1,
          index as u64,
          total as u64,
          compress_path.to_string(),
          1,
          file_name.clone());
        println!("压缩文件：{}失败：{}", name.to_str().unwrap(), e.to_string());
        continue;
      }
      let open_result = File::open(path);
      if let Err(e) = open_result{
        //发送失败消息到socket中
        send_to_socket(
          1,
          index as u64,
          total as u64,
          compress_path.to_string(),
          1,
          file_name.clone());
        println!("压缩文件：{}失败：{}", name.to_str().unwrap(), e.to_string());
        continue;
      }

      let mut f = open_result.unwrap();
      let read_result = f.read_to_end(&mut buffer);
      if let Err(e) = read_result{
        //发送失败消息到socket中
        send_to_socket(
          1,
          index as u64,
          total as u64,
          compress_path.to_string(),
          1,
          file_name.clone());
        println!("压缩文件：{}失败：{}", name.to_str().unwrap(), e.to_string());
        continue;
      }

      let write_result = zip.write_all(&*buffer);
      if let Err(e) = write_result {
        //发送失败消息到socket中
        send_to_socket(
          1,
          index as u64,
          total as u64,
          compress_path.to_string(),
          1,
          file_name.clone());
        println!("压缩文件：{}失败：{}", name.to_str().unwrap(), e.to_string());
        continue;
      }

      buffer.clear();
    } else if name.as_os_str().len() != 0 {//目录
      #[allow(deprecated)]
      let add_result = zip.add_directory_from_path(name, options);
      match add_result {
        Ok(_) => {}
        Err(e) => {
          //发送失败消息到socket中
          send_to_socket(
            1,
            index as u64,
            total  as u64,
            compress_path.to_string(),
            1,
            file_name.clone());
          println!("压缩文件：{}失败：{}", name.to_str().unwrap(), e.to_string());
          continue;
        }
      }
    }
    //发送消息到socket中
    send_to_socket(
      1,
      index as u64,
      total as u64,
      compress_path.to_string(),
      0,
      file_name.clone());
    index = index +1;
  }
  zip.finish()?;
  Result::Ok(())
}


///解压
/// test.zip文件解压到d:/test文件夹下
fn extract(test: &SPath, target: &SPath) {
  if let Err(e) = std::fs::File::open(&test) {
    let mut msg = e.to_string();
    msg.push_str(test.to_str().unwrap());
    println!("解压出错：{}", e);
  }
  let file_name = test.clone().file_name().unwrap().to_str().unwrap().to_string();
  let zipfile = std::fs::File::open(&test).unwrap();
  let mut zip = zip::ZipArchive::new(zipfile).unwrap();
  //socket 发送的路径
  let send_path = target.clone();
  //是否成功的标示
  let mut handle_result = 0;
  let total_number = zip.len();
  for i in 0..zip.len() {
    let mut file = zip.by_index(i).unwrap();
    if file.is_dir() {
      //println!("file utf8 path {:?}", file.name_raw());//文件名编码,在windows下用winrar压缩的文件夹，中文文夹件会码(发现文件名是用操作系统本地编码编码的，我的电脑就是GBK),本例子中的压缩的文件再解压不会出现乱码
      let target = target.join(SPath::new(&file.name().replace("\\", "")));
      fs::create_dir_all(target).unwrap();
    } else {
      let file_path = target.join(SPath::new(file.name()));
      let mut target_file = if !file_path.exists() {
        fs::File::create(file_path).unwrap()
      } else {
        fs::File::open(file_path).unwrap()
      };
      let result = copy(&mut file, &mut target_file);
      if let Err(e) = result {
        handle_result = 1;
        println!("解压文件:{} 失败:{}", file.name(), e.to_string());
      }

      // target_file.write_all(file.read_bytes().into());
      //发送消息到socket中
      send_to_socket(
        0,
        (i+1) as u64,
        total_number as u64,
        send_path.to_str().unwrap().to_string(),
        handle_result,
        file_name.clone());

    }
  }
}

/// 发送消息到socket
fn send_to_socket(handle_type:u8,current_index:u64,total_number:u64,handle_path:String,handle_result: u8,file_name:String) {
  if GLOBAL_SOCKET.lock().unwrap().user_set.lock().unwrap().len() > 0 {
    let socket_data = FileSocketData::new(
      handle_type,
      current_index,
      total_number,
      handle_path,
      handle_result,
      file_name
    );
    //转json
    let data = serde_json::to_string(&socket_data).unwrap();
    GLOBAL_SOCKET.lock().unwrap().tx.send(String::from(data));
  }
}

//使用websocket推送消息
pub async fn websocket_handle(ws: WebSocketUpgrade,state:Arc<FileAppState>,args:Query<HashMap<String,String>>)->impl IntoResponse {
    ws.on_upgrade(|socket| handle(socket,args,state))
}

async fn handle(mut socket: WebSocket,args:Query<HashMap<String,String>>,stat:Arc<FileAppState>){
  if let None = args.0.get("id"){
    let err =String::from("请传入ID");
    let error:RespVO<String> = RespVO::from_error(err,String::from(""));
    let msg = serde_json::to_string(&error).unwrap();
    //发送错误提示
    socket.send(Message::Text(msg))
        .await;
    return;
  }
  let id: &String = args.0.get("id").unwrap();

  let ( sender,  receiver) = socket.split();

  stat.user_set.lock().unwrap().insert(id.clone());

  let mut read_tack = tokio::spawn(read(receiver,id.clone()));

  let sd = stat.tx.subscribe();

  //发送消息
  let mut write_tack = tokio::spawn(write(sender, sd));

  // 当发送消息失败或者离开页面， 阻塞方法
  tokio::select! {
         _ = (&mut write_tack) => read_tack.abort(),
        _ = (&mut read_tack) => write_tack.abort(),
    };

}

async fn read(mut receiver: SplitStream<WebSocket>,id:String) {
  // ...
  while let Some(Ok(message))=receiver.next().await{
    match message {
      Message::Text(msg) => {
        println!("{}",msg);
      }
      Message::Close(c) => {
        //移出socket
        GLOBAL_SOCKET.lock().unwrap().user_set.lock().unwrap().remove(&*id);
      }
      _ => {
        println!("其它消息")
      }
    }
  }
}

async fn write(mut sender: SplitSink<WebSocket, Message>, mut sd: Receiver<String>) {
  while let Ok(message) = sd.recv().await{
    if sender.send(Message::Text(message)).await.is_err(){
      break;
    }
  }
}
