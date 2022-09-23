use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct FileEntity{
    file_name:String,
    // 文件大小 byte
    file_size:u64,
    //更新时间 毫秒
    update_date:u128,
    // 0 文件夹 1 文件
    file_type:u8,
}

impl FileEntity {
    pub fn new(file_name:String, file_size:u64, update_date:u128,file_type:u8) -> FileEntity {
        FileEntity{
            file_name,
            file_size,
            update_date,
            file_type,
        }
    }
}


// 文件管理模块 socket 管理器
pub struct FileAppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

impl FileAppState {
    pub fn new() -> Arc<FileAppState> {
        let user_set = Mutex::new(HashSet::new());
        let (tx, _rx) = broadcast::channel(100);
        Arc::new(FileAppState { user_set, tx })
    }
}

///socket推送消息解构
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct FileSocketData{
    //处理类型 0 解压 1 压缩
    handle_type:u8,
    //当前文件索引
    current_index:u64,
    //总文件数
    total_number:u64,
    //当前处理路径
    handle_path:String,
    //处理结果 0成功 1失败
    handle_result:u8,
    //压缩或解压包的名字
    file_name:String,
}

impl FileSocketData{
    pub fn new(handle_type: u8,current_index:u64,total_number:u64,handle_path:String,handle_result:u8,file_name:String) ->FileSocketData{
        FileSocketData{
            handle_type,
            current_index,
            total_number,
            handle_path,
            handle_result,
            file_name
        }
    }
}