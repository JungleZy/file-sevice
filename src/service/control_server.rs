use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use axum::{response::IntoResponse, extract::{ContentLengthLimit, Multipart}, http::HeaderMap, TypedHeader, headers};
use common::RespVO;
use std::process::{Command, Output, Stdio};
use std::os::windows::process::CommandExt;

use axum::{Error, extract::ws::{WebSocket, Message}};
use axum::response::Response;
use chrono::Local;
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, System as s_System, SystemExt};



use serde::{Serialize, Deserialize};
use http_server::INFO;

use tokio::sync::broadcast;
// socket 管理器
pub struct AppState {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
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
struct DiskDetail{
    name:String,
    total:u64,
    usable:u64,
    used:u64,
    ratio:f64,

}
impl DiskDetail {
    fn generate()->DiskDetail{
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
    cpu_name:String,
    //操作系统
    pub(crate) system_info:String,
    //内存大小
    memory_size:u64,
    //rust 版本
    rust_version:String,
    //运行时长
    run_time:String,
    //服务器版本
    server_version:String,
    //当前磁盘
    current_disk:String,
}
impl SystemInfo {
     fn generate()->SystemInfo{
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
struct ChartInfo {
    rtmp_connect:u32,
    http_connect:u32,
    web_socket_connect:u32,
    cpu_usage:u64,
    net_send:u64,
    net_received:u64,
    memory_total:u64,
    memory_used:u64,
}
impl ChartInfo {

    fn generate()->ChartInfo{
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
struct ControlInfo {
    server_info:Option<SystemInfo>,
    char_info:Option<ChartInfo>,
    disk_detail:Option<Vec<DiskDetail>>,
}

impl ControlInfo {
    fn generate()->ControlInfo{
        ControlInfo{
            server_info: None,
            char_info: None,
            disk_detail: None
        }
    }
}


/*pub async fn file_info() -> impl IntoResponse {
    let s = "File Service".to_string();
    return RespVO::from(&s).resp_json();
}*/



use crate::http::http_server;
use crate::http::http_server::START_TIME;


pub  async fn server_info()-> impl IntoResponse{
    let disk_info = get_disk_info();
    let server_info = get_server_info();
    let chart_info = get_chart_info();
    let mut control_info = ControlInfo::generate();
    control_info.char_info  = Some(chart_info);
    control_info.server_info = Some(server_info);
    control_info.disk_detail = Some(disk_info);

    let ret = serde_json::to_string(&control_info).unwrap();

    return RespVO::from(&control_info).resp_json();
}
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::extract::{Query, WebSocketUpgrade};
use once_cell::sync::{Lazy, OnceCell};
use futures::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};
use tokio::sync::broadcast::Receiver;

//初始化全局socket线程管理器
pub static GLOBAL_SOCKET:Lazy<Mutex<Arc<AppState>>> = Lazy::new(||{
    let app_state = AppState::new();
    Mutex::new(app_state)
});

//WebSocket处理函数
pub async fn server_info_socket(ws: WebSocketUpgrade,args:Query<HashMap<String,String>>,stat:Arc<AppState>)->impl IntoResponse{

    ws.on_upgrade(|socket| handle_socket(socket,args,stat))

}

async fn handle_socket(mut socket: WebSocket,args:Query<HashMap<String,String>>,stat:Arc<AppState>) {
    let id: &String = args.0.get("id").unwrap();
    let (mut sender, mut receiver) = socket.split();

    stat.user_set.lock().unwrap().insert(id.clone());

    let mut read_tack = tokio::spawn(read(receiver, id.clone()));
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


//定时读取PC信息并发送
pub async fn read_os_info_to_send_socket(){
    loop {
        let disk_info = get_disk_info();
        let server_info = get_server_info();
        let chart_info = get_chart_info();
        let mut control_info = ControlInfo::generate();
        control_info.char_info = Some(chart_info);
        control_info.server_info = Some(server_info);
        control_info.disk_detail = Some(disk_info);
        let ret = serde_json::to_string(&control_info).unwrap();
        if GLOBAL_SOCKET.lock().unwrap().user_set.lock().unwrap().len() > 0{
            GLOBAL_SOCKET.lock().unwrap().tx.send(ret);
        }
        //停止2秒钟
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}


//获取图表各项统计信息
fn get_chart_info()->ChartInfo{
    let mut char_info = ChartInfo::generate();
    let mut  sys = s_System::new_all();
    sys.refresh_all();
    //设置网络接收和发送
    for (interface_name, data) in sys.networks() {
        if interface_name == "WLAN"{
            char_info.net_send  = data.transmitted();
            char_info.net_received = data.received();
        }
    }
    sys.refresh_memory();
    char_info.memory_total = sys.total_memory();
    char_info.memory_used = sys.used_memory();
    sys.refresh_cpu();
    char_info.cpu_usage = sys.global_cpu_info().cpu_usage() as u64;

    return char_info;

}

//读取磁盘信息
fn get_disk_info() -> Vec<DiskDetail> {

    let output = windows_cmd("wmic logicaldisk list brief");
    let disk_list = String::from_utf8_lossy(&output.stdout);
    let mut split = disk_list.split("\r\r\n");
    let mut flag = true;
    let mut num = 1;
    let mut ret = Vec::new();

    while flag {
        match split.next() {
            Some(s) => {
                if num != 1 && s != "" {
                    let mut disk_detail = DiskDetail::generate();

                    let mut info = ["".to_string(), "".to_string(), "".to_string(), "".to_string()];
                    let mut vec = Vec::new();

                    // C:        3          21034897408                127044808704  Windows
                    let mut clear = false;
                    let mut value = String::new();
                    let mut count = 0;
                    for x in s.chars() {
                        if x.to_string() != " " {
                            value.push(x);
                            clear = false;
                        } else {
                            if !clear {
                                clear = true;
                                // info[count] = value;
                                vec.push(value);
                                value = String::new();
                                count = count + 1;
                            }
                        }
                    }

                    match vec.get(0) {
                        None => {}
                        Some(t) => { disk_detail.name = t.to_string(); }
                    }

                    match vec.get(2) {
                        None => {}
                        Some(t) => { disk_detail.usable = t.to_string().parse().unwrap(); }
                    }

                    match vec.get(3) {
                        None => {}
                        Some(t) => { disk_detail.total = t.to_string().parse().unwrap(); }
                    }


                    //计算已使用，和占比

                    disk_detail.used = disk_detail.total - disk_detail.usable;

                    let ratio = (disk_detail.used as f64 / disk_detail.total as f64 * 100.0).to_string();
                    let x1:f64 = ratio[0..5].parse().unwrap();
                    disk_detail.ratio = x1 as f64;

                    ret.push(disk_detail);
                    // println!("{:?}",disk_detail);
                } else {
                    num = 0;
                }
            }
            _ => { flag = false; }
        }
    }

    ret
}

//获取服务器信息
pub  fn get_server_info() ->SystemInfo{
    let mut si = SystemInfo::generate();
    let cpu_info = get_cpu_info();
    let cpu_option = cpu_info.split_once("*");
    //cpu型号
    let cpu_name = cpu_option.unwrap().0;
    //cpu 说明 Intel64 Family 6 Model 158 Stepping 10
    let cpu_caption = cpu_option
        .unwrap().1
        .split_once(" ")
        .unwrap().0;

    let cpu_number = &cpu_caption[cpu_caption.len()-2..];

    unsafe {
        match &INFO {
            Some(s) => {
                let temp = s.to_string();
                si.system_info = temp;
            }
            None => {
                let os_info = get_os_info(cpu_number);
                &si.system_info.push_str(&os_info);
                let mut temp: String = String::from("x");
                temp.push_str(&os_info);
                INFO = Some(temp);
            }
        }
        //运行时长
        si.run_time = (Local::now().timestamp_millis() - START_TIME).to_string();
    }


    //获取内存大小
    si.memory_size = get_memory_size();

    si.cpu_name = cpu_name.to_string();
    si.current_disk = get_current_application_disk();
    si.rust_version = get_rust_version();

    si

}

//获取rust版本
fn get_rust_version()->String{
    let output = windows_cmd("rustc --version");
    //cargo 1.62.1 (a748cf5a3 2022-06-08)
    let cow = String::from_utf8_lossy(&output.stdout);
    let string = cow.to_string();
    let x = string.split_once(" ")
        .unwrap()
        .1;
    let i = x.find(" ").unwrap();
    let v = &x[..i];
    return  v.to_string();
}

//获取内存大小
fn get_memory_size() -> u64 {
    let output = windows_cmd("wmic memorychip list brief");
    let cow = String::from_utf8_lossy(&output.stdout);
    let x:Vec<&str> = cow.trim().split("\r\r\n").collect();
    let mut total_size:u64 = 0;
    for i in 0..x.len() {
        if i !=0 {
            let t:Vec<&str> = x.get(i)
                .unwrap()
                .split("  ")
                .collect();
            let size:u64 = t.get(0).unwrap().parse().unwrap();
            total_size = total_size+size;
        }
    }
    total_size /1024 / 1024 /1024
}

//读取os操作系统
fn get_os_info(cpu_number: &str) ->String{
    let output = windows_cmd("systeminfo");
    let cow = String::from_utf8_lossy(&output.stdout);
    let split = cow.split("\r\n");
    let mut ret = String::new();

    let mut os_version = "";
    let mut win_type = "";
    for x in split {
        let option = x.split_once(":");
        match option {
            Some(s) => {
                //操作系统位数
                if s.0 == "System Type" {
                    if s.1.trim() == "x64-based PC"{
                        win_type = ("_win64_");

                    }else {
                        win_type= ("_win32_");
                    }
                }
                //版本信息
                else if s.0 == "OS Version"{
                    let version = s.1.trim()
                        .split_once(" ")
                        .unwrap();
                    os_version = version.0;


                }

            }

            None => {}
        }
    }
    ret.push_str(cpu_number);
    ret.push_str(win_type);
    ret.push_str(os_version);
    return ret;
}

//获取当前程序所在的磁盘
fn get_current_application_disk() -> String {
    // windows_cmd()
    let result = std::env::current_dir();
    result.unwrap()
        .to_string_lossy()
        .split_once("\\")
        .unwrap()
        .0
        .to_string()
}

//读取pc信息 Intel(R) Core(TM) i5-9400 CPU @ 2.90GHz * Intel64 Family 6 Model 158 Stepping 10
fn get_cpu_info()->String{


    let cow = windows_cmd("wmic cpu list brief");

    let cpu_info = String::from_utf8_lossy(&cow.stdout);

    let mut cpu_info_format = cpu_info.split("\r\r\n");


    let mut  flag = true;
    let mut count = 1;

    let mut index:i32 = 0;

    let mut cpu = String::new();

    while flag {
        match cpu_info_format.next() {
            Some(s) => {
                if count ==1{
                    let vec1 = str_format(s);
                    //找到Name的位置
                    for i in 0..vec1.len() {
                        if vec1.get(i).unwrap() .eq("Name"){
                            index = i as i32;
                        }
                    }
                }else {
                    let split = s.split("  ");
                    let mut values = Vec::new();
                    for x in split {
                        if x != "" {
                            values.push(x.trim());
                        }
                    }

                    cpu = values.get(index as usize).unwrap().to_string();
                    cpu.push_str("*");
                    cpu.push_str(values.get(0).unwrap());

                    return cpu;
                }

                count = count+1;
            }
            None => {flag=false;}
        }
    }
    return cpu;


}


//将控制台的内容格式化
fn str_format(s:&str)->Vec<String>{
    let mut vec = Vec::new();
    let mut value = String::new();
    let mut flag = false;
    for x in s.chars() {
        if x.to_string() != " "{
            value.push(x);
            flag = true;
        }else {
            if flag {
                //1.将 group 放入到 vec中
                &vec.push(value.to_string());
                //2.清除内容
                value = String::new();
                //3.标记重置
                flag = false;

            }
        }
    }
    vec
}

//dos 命令
fn windows_cmd(command: &str) -> Output {
    let output = Command::new("cmd").creation_flags(0x08000000).arg("/c").arg(command)
        .stdout(Stdio::piped()).output().expect("cmd exec error!");
   return  output;
}





