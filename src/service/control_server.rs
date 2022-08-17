use std::fs;
use axum::{response::IntoResponse, extract::{ContentLengthLimit, Multipart}, http::HeaderMap};
use common::RespVO;
use std::process::{Command,Stdio};
use std::os::windows::process::CommandExt;


use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
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

/*pub async fn file_info() -> impl IntoResponse {
    let s = "File Service".to_string();
    return RespVO::from(&s).resp_json();
}*/



pub  async fn server_info()-> impl IntoResponse{
    let res_json = get_disk_info();

    return RespVO::from(&res_json).resp_json();
}

//读取磁盘信息
fn get_disk_info() -> String {

    let output = Command::new("cmd").creation_flags(0x08000000).arg("/c").arg(" wmic logicaldisk list brief")
        .stdout(Stdio::piped()).output().expect("cmd exec error!");

    let disk_list = String::from_utf8_lossy(&output.stdout);
    let mut split = disk_list.split("\r\r\n");
    // println!("{:?}",split);
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
                        Some(t) => { disk_detail.name = t.parse().unwrap(); }
                    }

                    match vec.get(2) {
                        None => {}
                        Some(t) => { disk_detail.usable = t.parse().unwrap(); }
                    }

                    match vec.get(3) {
                        None => {}
                        Some(t) => { disk_detail.total = t.parse().unwrap(); }
                    }


                    //计算已使用，和占比

                    disk_detail.used = disk_detail.total - disk_detail.usable;

                    disk_detail.ratio = (disk_detail.used as f64 / disk_detail.total as f64 * 100.0);

                    ret.push(disk_detail);
                    // println!("{:?}",disk_detail);
                } else {
                    num = 0;
                }
            }
            _ => { flag = false; }
        }
    }

    let res_json = serde_json::to_string(&ret).unwrap();
    res_json
}




