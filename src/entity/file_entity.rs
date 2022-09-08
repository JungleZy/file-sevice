use serde::{Serialize, Deserialize};

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