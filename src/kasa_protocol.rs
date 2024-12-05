use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::io::prelude::*;
use std::net::TcpStream;

use crate::models::{KasaChildren, KasaResp, Realtime, SysInfo};

// https://github.com/softScheck/tplink-smartplug/blob/master/tplink_smartplug.py#L70
pub fn encrypt(input: &str, inc_len: bool) -> Vec<u8> {
    let mut key = 171; //just the initial key
    let mut result: Vec<u8> = vec![];

    //the same xor is used for both discovery and the protocol
    //discovery lacks it, so we'll just switch here
    if inc_len {
        for octet in (input.len() as u32).to_be_bytes() {
            result.push(octet);
        }
    }

    for c in input.bytes() {
        let a = key ^ c;
        key = a;
        result.push(a);
    }
    return result;
}

pub fn decrypt(input: &Vec<u8>) -> String {
    let mut key = 171;
    let mut result = String::new();

    for c in input {
        let a = key ^ *c;
        key = *c;
        result.push(a as char);
    }

    return result;
}

pub fn deserialize(input: &str) -> KasaResp {
    let s: KasaResp = serde_json::from_str(input).expect("deserialization failed");
    return s;
}

pub fn read_kasa_resp(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut len = [0u8; 4];
    let _ = stream.read_exact(&mut len); //TODO: add a timeout here and return an option or result
    let len: usize = u32::from_be_bytes(len).try_into().unwrap();
    //println!("resp len: {}", len);

    let mut recv: Vec<u8> = vec![];
    let mut rx_bytes = [0u8; 256];
    loop {
        let bytes_read = match stream.read(&mut rx_bytes) {
            Ok(bytes) => bytes,
            Err(err) => {
                println!("stream.read failed, err: {:}", err);
                return Err(anyhow!("stream read error: {:?}", err));
            }
        };

        recv.extend_from_slice(&rx_bytes[..bytes_read]);
        if recv.len() >= len {
            break;
        }
    }
    return Ok(recv);
}

pub fn send_kasa_cmd(stream: &mut TcpStream, cmd: &str) {
    let cmd = encrypt(cmd, true);
    let _ = stream.write(&cmd);
}

fn send_and_read(stream: &mut TcpStream, cmd: &str) -> Result<KasaResp> {
    send_kasa_cmd(stream, &cmd);
    let resp = read_kasa_resp(stream)?;
    let resp: KasaResp = deserialize(&decrypt(&resp));
    return Ok(resp);
}

pub fn get_sys_info(stream: &mut TcpStream) -> Result<SysInfo> {
    let cmd = r#"{"system":{"get_sysinfo":null}}"#;
    send_kasa_cmd(stream, &cmd);
    let resp = read_kasa_resp(stream)?;
    let resp: KasaResp = deserialize(&decrypt(&resp));
    if let Some(system) = resp.system {
        if let Some(sys_info) = system.get_sysinfo {
            return Ok(sys_info);
        }
    }
    return Err(anyhow!("failed to get sys_info"));
}

//pub fn get_all_realtime_mt(stream: &mut TcpStream) -> Option<Vec<Realtime>> {
//    let c = get_children(stream)?;
//    //you'd think a field with a plural 'ids' in list brackets would accept a list
//    // you'd be wrong, so we're calling it multiple times, otherwise it only returns idx0
//    let ids:Vec<String> = c.into_iter().map(|x| x.id).collect();
//    let mut rts: Arc<Mutex<Vec<Realtime>>> = Arc::new(Mutex::new(vec![]));
//    let mut threads = vec![];
//
//
//
//    for id in ids {
//        threads.push(thread::spawn( move || -> Option<()> {
//
//            let resp: KasaResp = send_and_read( stream,
//                 &json!({
//                     "context" : {
//                         "child_ids" : [ id ]
//                 },
//                 "emeter": {
//                     "get_realtime":null
//                 },
//
//                 }).to_string()
//             )?;
//
//
//            match rts.lock() {
//                Ok(rt) => { rt.push(resp.emeter?.get_realtime?);}
//                _ => (),
//            }
//            Some(())
//
//        }));
//
//    }
//    for thread in threads {
//        thread.join();
//    }
//
//    let ret = Arc::into_inner(rts)?.into_inner().ok()?;
//    return Some(ret);
//
//
//}

pub fn get_all_realtime(stream: &mut TcpStream) -> Result<Vec<Realtime>> {
    let c = get_children(stream)?;
    //you'd think a field with a plural 'ids' in list brackets would accept a list
    // you'd be wrong, so we're calling it multiple times, otherwise it only returns idx0
    let ids: Vec<String> = c.into_iter().map(|x| x.id).collect();
    let mut rts: Vec<Realtime> = vec![];

    for id in ids {
        let resp: KasaResp = send_and_read(
            stream,
            &json!({
                "context" : {
                    "child_ids" : [ id ]
            },
            "emeter": {
                "get_realtime":null
            },

            })
            .to_string(),
        )?;
        if let Some(emeter) = resp.emeter {
            if let Some(rt) = emeter.get_realtime {
                rts.push(rt)
            }
        }
        //rts.push(resp.emeter?.get_realtime?)
    }

    return Ok(rts);
}

pub fn get_realtime_by_id(stream: &mut TcpStream, id: &String) -> Result<Realtime> {
    let resp: KasaResp = send_and_read(
        stream,
        &json!({
            "context" : {
                "child_ids" : [ id ]
        },
        "emeter": {
            "get_realtime":null
        },

        })
        .to_string(),
    )?;

    //let rt = resp.emeter?.get_realtime?;
    if let Some(emeter) = resp.emeter {
        if let Some(rt) = emeter.get_realtime {
            return Ok(rt);
        }
    }
    return Err(anyhow!("Realtime Response is None"));
}

pub fn get_realtime(stream: &mut TcpStream) -> Result<Realtime> {
    let cmd = r#"{"emeter":{"get_realtime":null}}"#;
    send_kasa_cmd(stream, &cmd);
    let resp = read_kasa_resp(stream)?;
    let resp: KasaResp = deserialize(&decrypt(&resp));
    if let Some(emeter) = resp.emeter {
        if let Some(rt) = emeter.get_realtime {
            return Ok(rt);
        }
    }
    return Err(anyhow!("Realtime response is none"));
}

pub fn get_realtime_by_idx(stream: &mut TcpStream, idx: usize) -> Result<Realtime> {
    let children = get_children(stream)?;
    if idx > children.len() {
        return Err(anyhow!(
            "invalid idx: {idx} where n children: {}",
            children.len()
        ));
    }
    get_realtime_by_id(stream, &children[idx].id)
}

pub fn get_children(stream: &mut TcpStream) -> Result<Vec<KasaChildren>> {
    let c: Vec<KasaChildren> = get_sys_info(stream)?.children;
    return Ok(c);
}

pub fn toggle_relay_by_alias(_stream: &mut TcpStream, _alias: String) {
    return;
}

pub fn toggle_relay_by_idx(stream: &mut TcpStream, idx: usize) -> Result<bool> {
    let children = get_children(stream).unwrap();
    if idx < children.len() {
        let child_id = &children[idx].id;
        let state = match children[idx].state {
            0 => 1,
            _ => 0,
        };
        let result = set_relay_by_child_id(stream, &child_id, state);
        return result;
    }
    return Err(anyhow!("Invaid child idx: {}", idx));
}

pub fn toggle_single_relay_outlet(stream: &mut TcpStream) -> Result<bool> {
    let state = match get_sys_info(stream)?.relay_state {
        0 => 1,
        _ => 0,
    };
    set_single_relay_outlet(stream, state)
}

pub fn set_single_relay_outlet(stream: &mut TcpStream, state: u8) -> Result<bool> {
    let cmd: String = json!({
        "system": {
            "set_relay_state" : {
                "state" : state
            }
        }
    })
    .to_string();

    send_kasa_cmd(stream, cmd.as_str());
    let resp: Value = serde_json::from_str(&decrypt(&read_kasa_resp(stream).unwrap()))?;

    let err_resp = resp["system"]["set_relay_state"]["err_code"]
        .as_i64()
        .unwrap();

    return Ok(match err_resp {
        0 => false,
        _ => true,
    });
}

pub fn set_relay_by_child_id(stream: &mut TcpStream, child_id: &str, state: u8) -> Result<bool> {
    let cmd: String = json!({
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
    let resp: Value = serde_json::from_str(&decrypt(&read_kasa_resp(stream).unwrap()))?;

    let err_resp = resp["system"]["set_relay_state"]["err_code"]
        .as_i64()
        .unwrap();

    return Ok(match err_resp {
        0 => false,
        _ => true,
    });
}

pub fn set_outlet_alias(stream: &mut TcpStream, child_id: &str, alias: &str) -> Result<bool> {
    let cmd: String = json!({
        "context" : {
            "child_ids": [ child_id ]
        },
        "system" : {
            "set_dev_alias":{
                "alias": alias
            }
        }
    })
    .to_string();

    send_kasa_cmd(stream, cmd.as_str());
    let resp: Value = serde_json::from_str(&decrypt(&read_kasa_resp(stream).unwrap()))?;

    let err_resp = resp["system"]["set_dev_alias"]["err_code"]
        .as_i64()
        .unwrap();
    println!("{err_resp}");

    return Ok(match err_resp {
        0 => false,
        _ => true,
    });
}
