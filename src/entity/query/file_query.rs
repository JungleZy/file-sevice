use serde::{Deserialize};
// 移出文件参数
#[derive(Deserialize)]
pub struct RemoveFileQuery{
    pub path:String,
    pub is_file:bool,
}

//压缩文件参数
#[derive(Deserialize)]
pub struct CompressedFilesParam{
    //目录
    pub path:String,
    //文件
    pub files:Vec<String>,
    //压缩的文件名
    pub zip_name:String,
}