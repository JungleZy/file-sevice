
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{copy, Read, Seek, Write};
use std::path::Path as SPath;
use std::time::UNIX_EPOCH;
use std::vec::IntoIter;
use axum::{response::IntoResponse, extract::{ContentLengthLimit, Multipart}, http::HeaderMap, Json};
use axum::body::Body;
use axum::extract::Query;
use axum::http::Response;
use common::RespVO;
use rand::random;
use walkdir::{DirEntry, WalkDir};

use zip::write::FileOptions;
use crate::entity::file_entity::FileEntity;
use crate::entity::query::file_query::{CompressedFilesParam, RemoveFileQuery, UnCompressedFileParam};

const SAVE_FILE_BASE_PATH: &str = ".\\file";

//获取文件列表
pub async fn file_info(query:Query<HashMap<String,String>>) -> impl IntoResponse {
  let option = query.0.get("path");
  let mut show_path = SAVE_FILE_BASE_PATH.to_string();
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
          // println!("文件大小：{}", );
          // println!("{}", );

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
      let content_type = field.content_type().unwrap().to_string();
      let data = field.bytes().await.unwrap();
      
      let current_path = headers.get("currentPath");
      let mut upload_path = "";
      if current_path.is_none() == false {
        upload_path = current_path.unwrap().to_str().unwrap();
      }
      let mut save_path =  SAVE_FILE_BASE_PATH.to_string();
      if upload_path != "" {
        save_path = format!("{}/{}", SAVE_FILE_BASE_PATH, upload_path);
      }
      // fs::create_dir_all(save_path.to_string()).unwrap();
      println!(
          "Length of `{}` (`{}`: `{}`) is {} bytes",
          name,
          file_name,
          content_type,
          data.len()
      );
      // if content_type.starts_with("image/") {
        //根据文件类型生成随机文件名(出于安全考虑)
        let rnd = (random::<f32>() * 1000000000 as f32) as i32;
        //提取"/"的index位置
        /*let index = content_type
            .find("/")
            .map(|i| i)
            .unwrap_or(usize::max_value());*/
        //文件扩展名
        let ext_name;
       /* if index != usize::max_value() {
            ext_name = &content_type[index + 1..];
        }*/
        // 文件后缀.的位置
        if let Some(ext_index) = file_name.rfind(".") {
          ext_name = &file_name[ext_index+1..];
        }else {
          ext_name = "";
        }


      //最终保存在服务器上的文件名
        let save_filename = format!("{}/{}.{}", save_path, rnd, ext_name);

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

        return RespVO::from(&format!("/{}/{}.{}",upload_path, rnd, ext_name)).resp_json();
      // }
    }
    
    return RespVO::from(&msg).resp_json();
}


//删除文件夹或文件 path:&str,is_file:bool
pub async fn remove_dir_or_file(Json(query):Json<RemoveFileQuery>) -> impl IntoResponse{
  let mut full_path = SAVE_FILE_BASE_PATH.to_string();
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


//下载文件
pub async fn down_load_file(file_name:Query<HashMap<String,String>>) -> impl IntoResponse {
  let name = file_name.0.get("fileName");
  if let None = name{
    return  RespVO::from_error(String::from("请传入下载文件路径名称"),String::from(" ")).resp_json();
  }
  let name = name.unwrap();
  let mut file_name = String::from("attachment;filename=");
  if name.contains("/"){
    let v:Vec<&str> = name.rsplit("/").collect();
    let x = v.get(0).unwrap();
    println!("{:?}",v);
    file_name.push_str(x);
  }else {
    file_name.push_str(name);
  }
  let mut file_path = String::from(SAVE_FILE_BASE_PATH.clone());
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
    zip_path.push_str("/");
    zip_path.push_str(param.path.as_str());
    zip_path.push_str("/");
    zip_path.push_str(param.zip_name.as_str());
    let mut compressed_path = SAVE_FILE_BASE_PATH.to_string();
    compressed_path.push_str("/");
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

  let result = zip_dir(new_dir_entry.into_iter(),src_dir.to_str().unwrap(),zipfile);

  if let Err(e) = result{
    println!("{}",e.to_string());
    //return RespVO::from_error(e.to_string(),String::from("")).resp_json();
  }
}

fn zip_dir<T>(it: IntoIter<DirEntry>, prefix: &str, writer: T) -> zip::result::ZipResult<()>
  where T: Write + Seek {
  let mut zip = zip::ZipWriter::new(writer);
  let options = FileOptions::default()
      .compression_method(zip::CompressionMethod::Bzip2)//直接用了bzip2压缩方式，其它参看枚举
      .unix_permissions(0o755);//unix系统权限

  let mut buffer = Vec::new();
  for entry in it {
    let path = entry.path();
    // println!("{}",path.clone().file_name().unwrap().to_str().unwrap());
    //zip压缩一个文件时，会把它的全路径当成文件名(在下面的解压函数中打印文件名可知)
    //这里是去掉目录前缀
    let name = path.strip_prefix(SPath::new(prefix)).unwrap();

    // Write file or directory explicitly
    // Some unzip tools unzip files with directory paths correctly, some do not!
    if path.is_file() {
      #[allow(deprecated)]
      zip.start_file_from_path(name, options)?;
      let mut f = File::open(path)?;
      f.read_to_end(&mut buffer)?;
      zip.write_all(&*buffer)?;
      buffer.clear();
    } else if name.as_os_str().len() != 0 {//目录
      #[allow(deprecated)]
      zip.add_directory_from_path(name, options)?;
    }
  }
  zip.finish()?;
  Result::Ok(())
}


///解压
/// test.zip文件解压到d:/test文件夹下
///
fn extract(test: &SPath, target: &SPath){
  if let Err(e)=std::fs::File::open(&test){
    let mut msg = e.to_string();
    msg.push_str(test.to_str().unwrap());
    println!("解压出错：{}",e);
  }
  let zipfile = std::fs::File::open(&test).unwrap();
  let mut zip = zip::ZipArchive::new(zipfile).unwrap();

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
      if let Err(e) = result{
        println!("解压文件:{} 失败:{}",file.name(),e.to_string());
      }
      // target_file.write_all(file.read_bytes().into());
    }
  }
}