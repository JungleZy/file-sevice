#![allow(unused_variables)] //允许未使用的变量
#![allow(dead_code)] //允许未使用的代码
#![allow(unused_must_use)]

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use serde::{Serialize, Deserialize};

// socket 管理器
pub struct AppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> Arc<AppState> {
        let user_set = Mutex::new(HashSet::new());
        let (tx, _rx) = broadcast::channel(100);
        Arc::new(AppState { user_set, tx })
    }
}


//磁盘信息
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct DiskDetail{
    pub name:String,
    pub total:u64,
    pub usable:u64,
    pub used:u64,
    pub ratio:f64,

}
impl DiskDetail {
    pub fn generate()->DiskDetail{
        DiskDetail{
            name: "".to_string(),
            total: 0,
            usable: 0,
            used:0,
            ratio:0.0,
        }
    }
}

//服务器信息
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct SystemInfo{
    //cpu名称
    pub cpu_name:String,
    //操作系统
    pub(crate) system_info:String,
    //内存大小
    pub memory_size:u64,
    //rust 版本
    pub rust_version:String,
    //运行时长
    pub run_time:String,
    //服务器版本
    pub server_version:String,
    //当前磁盘
    pub current_disk:String,
}
impl SystemInfo {
    pub(crate) fn generate() ->SystemInfo{
        SystemInfo{
            cpu_name: "".to_string(),
            system_info: "x".to_string(),
            memory_size: 0,
            rust_version: "".to_string(),
            run_time: "".to_string(),
            server_version: "1.0.0".to_string(),
            current_disk: "".to_string()
        }

    }

}

// 图表统计信息
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ChartInfo {
    rtmp_connect:u32,
    http_connect:u32,
    web_socket_connect:u32,
    pub(crate) cpu_usage:u64,
    pub(crate) net_send:u64,
    pub(crate) net_received:u64,
    pub(crate) memory_total:u64,
    pub(crate) memory_used:u64,
}
impl ChartInfo {

    pub(crate) fn generate() ->ChartInfo{
        ChartInfo{
            rtmp_connect: 0,
            http_connect: 0,
            web_socket_connect: 0,
            cpu_usage: 0,
            net_send: 0,
            net_received: 0,
            memory_total: 0,
            memory_used: 0
        }
    }

}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct WebSocketParam{
    id:String
}


//监控台
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ControlInfo {
    pub(crate) server_info:Option<SystemInfo>,
    pub(crate) char_info:Option<ChartInfo>,
    pub(crate) disk_detail:Option<Vec<DiskDetail>>,
}

impl ControlInfo {
    pub(crate) fn generate() ->ControlInfo{
        ControlInfo{
            server_info: None,
            char_info: None,
            disk_detail: None
        }
    }
}

