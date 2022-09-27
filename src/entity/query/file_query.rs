use serde::{Deserialize};
// 移出文件参数
#[derive(Deserialize)]
pub struct RemoveFileQuery{
    //路径
    pub path:String,
    //是否是文件
    pub is_file:bool,
    // 是否是 服务管理目录
    pub is_server_manage:bool,
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
    //是否是服务管理目录
    pub is_server_manage:bool,
}


//压缩文件参数
#[derive(Deserialize)]
pub struct UnCompressedFileParam{
    //目录
    pub path:String,
    //文件名称
    pub zip_name:String,
    //是否是服务管理目录
    pub is_server_manage:bool,
}