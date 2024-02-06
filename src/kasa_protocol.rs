use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::io::prelude::*;
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
pub struct NextAction {
    pub r#type: i32,
}
#[derive(Serialize, Deserialize)]
pub struct KasaChildren {
    pub id: String,
    pub state: u8, 
    pub alias: String,
    pub on_time: u64,
    pub next_action: NextAction,
}


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)] //kasa json mixes snake and camel and I don't have control of that
pub struct SysInfo {
    pub alias: String,
    pub child_num: usize,
    pub children: Vec<KasaChildren>,
    pub deviceId: String,
    pub err_code: u32,
    pub feature: String,
    pub hwId: String,
    pub hw_ver: String,
    pub latitude_i:u32,
    pub led_off:u8,
    pub longitude_i:u32,
    pub mac:String, 
    pub mic_type: String, 
    pub model:String,
    pub oemId:String,
    pub rssi: i32,
    pub status:String,
    pub sw_ver:String,
    pub updating:u32
}


#[derive(Serialize, Deserialize)]
pub struct System {
    pub get_sysinfo: SysInfo,
}


#[derive(Serialize, Deserialize)]
pub struct KasaResp {
    pub system: System,
}

// https://github.com/softScheck/tplink-smartplug/blob/master/tplink_smartplug.py#L70
pub fn encrypt(input: &str) -> Vec<u8> {
    let mut key = 171; //just the initial key
    let mut result : Vec<u8> = vec![];
    //let length = input.len().to_be_bytes();

    for octet in (input.len() as u32).to_be_bytes() {
        result.push(octet);
    }

    for c in input.bytes() {
        let a = key ^ c;
        key = a;
        result.push(a);
    }
    //println!("{:x?}", result);
    return result;
}

pub fn decrypt(input: &Vec<u8>) -> String {
    let mut key = 171;
    let mut result = String::new();
    
    //let len: [u8;4] = input[..4].try_into().unwrap();
    //let data = &input[4..];
    
    for c in input {
        let a = key ^ *c;
        key = *c;
        result.push(a as char);
    }
    //println!("len: {}, payload: {}", len, result);
    return result;
}
pub fn deserialize(input: &str) -> KasaResp {
    let s:KasaResp = serde_json::from_str(input)
        .expect("deserialization failed");
    return s
}

pub fn read_kasa_resp( stream: &mut TcpStream) -> Vec<u8> {
    let mut len = [0u8;4];
    let _ = stream.read_exact(&mut len);
    let len: usize = u32::from_be_bytes(len).try_into().unwrap();
    println!("resp len: {}", len);
    
    let mut recv: Vec<u8> = vec![];
    let mut rx_bytes = [0u8;256];
    loop {
        let bytes_read = stream.read(&mut rx_bytes).unwrap();
        recv.extend_from_slice(&rx_bytes[..bytes_read]);
        if recv.len() >= len {
            break;
        }
    }
    return recv
}

pub fn send_kasa_cmd( stream: &mut TcpStream,cmd: &str) {
    let cmd = encrypt(cmd);
    let _ = stream.write(&cmd);
}


pub fn get_sys_info(stream: &mut TcpStream) -> SysInfo {
    let cmd = r#"{"system":{"get_sysinfo":null}}"#;
    send_kasa_cmd(stream, &cmd);
    let resp = read_kasa_resp(stream);
    let resp: KasaResp = deserialize( &decrypt(&resp));

    return resp.system.get_sysinfo
}

pub fn get_children(stream: &mut TcpStream) -> Vec<KasaChildren> {
    let c : Vec<KasaChildren> =  get_sys_info(stream).children;
    return c
}

pub fn toggle_relay_by_alias(_stream: &mut TcpStream, _alias: String) {
    return
}

pub fn toggle_relay_by_idx(stream: &mut TcpStream, idx: usize) {
    let children = get_children(stream);
    if idx < children.len() {
        let child_id = &children[idx].id;
        let state = match children[idx].state {
             0 => 1,
             _ => 0,
        };
        let _ = set_relay_by_child_id(stream, &child_id, state);
    }


    return
}

pub fn set_relay_by_child_id(stream: &mut TcpStream, child_id: &str, state:u8) -> bool{

    let cmd:String = json!({
            "context" : {
                "child_ids": [ child_id ]
            },
            "system": {
                "set_relay_state" : {
                    "state" : state
                }
            }
        })
        .to_string();

    send_kasa_cmd(stream, cmd.as_str());
    let resp: Value = serde_json::from_str(
        &decrypt(
            &read_kasa_resp(stream)
        )
    ).unwrap();
    
    //let resp: Value = serde_json::from_str(&resp).unwrap();

    let err_resp = resp["system"]["set_relay_state"]["err_code"].as_i64().unwrap();

    return match err_resp {
        0 => false,
        _ => true,
    };
}



