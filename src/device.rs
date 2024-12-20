use crate::kasa_protocol::{
    self, decrypt, deserialize, encrypt, get_sys_info, set_relay_by_child_id, set_single_relay_outlet, toggle_relay_by_idx, toggle_single_relay_outlet, set_relay_by_idx
};
use crate::models::{KasaChildren, KasaResp, Realtime, SysInfo, System};
use crate::validate_ip;
use anyhow::{anyhow, Result};
use serde_json::json;
use std::io;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::time::Duration;

#[derive(Clone)]
pub struct Device {
    pub ip_addr: String,
    pub kasa_info: KasaResp,
    pub realtime: Vec<Realtime>,
}

impl Device {
    pub fn new(ip_addr: String, kasa_info: KasaResp) -> Device {
        Device {
            ip_addr,
            kasa_info,
            realtime: vec![],
        }
    }

    pub fn get_children(&self) -> Option<Vec<KasaChildren>> {
        let stream = TcpStream::connect(self.ip_addr.clone());
        if let Ok(mut strm) = stream {
            let children = match kasa_protocol::get_children(&mut strm) {
                Ok(chldrn) => chldrn,
                Err(_err) => vec![],
            };
            return Some(children);
        }
        //let children = self.kasa_info.system.unwrap().get_sysinfo.unwrap().children.clone();
        println!("failed to get children");
        return None;
    }

    pub fn get_realtime(&self) -> Option<Vec<Realtime>> {
        let stream = TcpStream::connect(self.ip_addr.clone());

        if let Ok(mut strm) = stream {
            let realtime = match kasa_protocol::get_all_realtime(&mut strm) {
                Ok(rt) => rt,
                Err(_err) => vec![],
            };
            return Some(realtime);
        }
        //let children = self.kasa_info.system.unwrap().get_sysinfo.unwrap().children.clone();
        println!("failed to get realtime");
        return None;
    }

    pub fn sysinfo_raw(&self) -> Option<String> {
        Some(serde_json::to_string(&self.kasa_info.system.clone()?.get_sysinfo?).unwrap())
    }

    pub fn sysinfo(&self) -> Option<SysInfo> {
        Some(self.kasa_info.system.clone()?.get_sysinfo?)
    }

    pub fn children(&self) -> Option<Vec<KasaChildren>> {
        Some(self.sysinfo()?.children)
    }
    pub fn num_children(&self) -> Option<usize> {
        Some(self.sysinfo()?.child_num)
    }
    pub fn has_children(&self) -> bool {
        if let Some(nc) = self.num_children() {
            if nc > 0 {
                return true;
            }
        }
        return false;
    }
    pub fn realtime(&self) -> Vec<Realtime> {
        self.realtime.clone()
    }

    pub fn toggle_relay_by_id(&self, idx: usize) {
        println!("ip:{:}, idx: {:}", self.ip_addr, idx);
        let stream = TcpStream::connect(self.ip_addr.clone());
        if let Ok(mut strm) = stream {
            let _ = toggle_relay_by_idx(&mut strm, idx);
            println!("toggl'd");
        }
    }
    //similar to kasa protocol but wont retrieve a new sysinfo first
    pub fn set_child_relay_by_idx(&self, idx: usize, state: u8) { 
        println!("{:}", state);
        if let Ok(mut stream) = TcpStream::connect(self.ip_addr.clone()) {
            if let Some(children) = self.children() {
                if idx < children.len() {
                    let child_id = &children[idx].id;
                    println!("{:}", child_id);
                    let _result = set_relay_by_idx(&mut stream, idx, state);
                    if let Ok(res) = _result {
                        println!("{:}", res);
                    }
                    println!("toggling");
                }
            }
        }
    }

    pub fn set_single_relay(&self, state: u8) {
        if let Ok(mut stream) = TcpStream::connect(self.ip_addr.clone()) {
            let _ = set_single_relay_outlet(&mut stream, state);
        }
    }

    pub fn toggle_single_relay(&self) {
        if let Ok(mut stream) = TcpStream::connect(self.ip_addr.clone()) {
            let _ = toggle_single_relay_outlet(&mut stream);
        }
    }
}

pub fn determine_target(t_addr: String) -> Result<Device> {
    if t_addr == "" {
        return Err(anyhow!("Discovery failed and no target was provided"));
        //try discovery
        //if let Ok(kd) = discover() {
        //    return Ok(kd);
        //} else {
        //    return Err(anyhow!("Discovery failed and no target was provided"));
        //}
    } else {
        if validate_ip(&t_addr) {
            println!("good ip");
            let mut stream = TcpStream::connect(t_addr.clone() + ":9999")?;
            return match get_sys_info(&mut stream) {
                Ok(si) => Ok(Device {
                    ip_addr: t_addr,
                    kasa_info: KasaResp {
                        system: Some(System {
                            get_sysinfo: Some(si),
                        }),
                        emeter: None,
                    },
                    realtime: vec![],
                }),
                Err(si) => Err(si),
            };
        } else {
            println!("bad ip");

            return Err(anyhow!("Provided ip is invalid"));
        }
    }
}

//this will only discover one
//will need to be revisited
//pub fn discover() -> Result<Device> {
//    let socket = UdpSocket::bind("0.0.0.0:46477")?;
//    socket.set_broadcast(true)?;
//
//    let cmd = json!({"system": {"get_sysinfo":0}}).to_string();
//    //this is for the newer devices, which I lack
//    //let cmd2 = "020000010000000000000000463cb5d3";
//
//    let enc_cmd = encrypt(&cmd, false);
//    //println!("{:?}", enc_cmd);
//    //let enc_cmd: &[u8] = &enc_cmd;
//
//    socket.send_to(&enc_cmd, "255.255.255.255:9999")?;
//
//    //println!("sent");
//
//    let mut buf = [0; 2048];
//
//    let mut ip_addr: String = String::new();
//
//    if let Ok((_n, addr)) = socket.recv_from(&mut buf) {
//        //println!("{} bytes response from {:?}", n, addr);
//        ip_addr = addr.to_string();
//    }
//
//    let mut len: usize = 0;
//    while buf[len] != 0 {
//        len += 1;
//    }
//    let mut recv: Vec<u8> = vec![];
//    recv.extend_from_slice(&buf[..len]);
//
//    let info = deserialize(&decrypt(&recv));
//    //println!("{}", info.system.unwrap().get_sysinfo.unwrap().sw_ver);
//
//    return Ok(Device::new(ip_addr, info));
//}

pub fn discover() -> Result<Vec<Device>> {
    let socket = UdpSocket::bind("0.0.0.0:46477")?;
    socket.set_broadcast(true)?;
    let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));

    let cmd = json!({"system": {"get_sysinfo":0}}).to_string();

    let enc_cmd = encrypt(&cmd, false);

    socket.send_to(&enc_cmd, "255.255.255.255:9999")?;

    let mut buf = [0; 2048];
    let mut devices: Vec<Device> = vec![];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, addr)) => {
                let ip_addr = addr.to_string();
                let mut recv: Vec<u8> = vec![];
                recv.extend_from_slice(&buf[..amt]);
                let info = deserialize(&decrypt(&recv));
                devices.push(Device::new(ip_addr, info));
                buf = [0; 2048];
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                //println!("Timed out");
                break;
            }
            Err(_) => {
                println!("something else");
                break;
            }
        }
    }

    return Ok(devices);
}

//Lower overhead version for embedded usecases.
//Won't allocate big buffers for full sysinfo, just throw
//it away and take the target ip
pub fn discover_ip() -> Result<Vec<String>> {
    let socket = UdpSocket::bind("0.0.0.0:46477")?;
    socket.set_broadcast(true)?;
    let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));

    let cmd = json!({"system": {"get_sysinfo":0}}).to_string();

    let enc_cmd = encrypt(&cmd, false);

    socket.send_to(&enc_cmd, "255.255.255.255:9999")?;

    let mut buf = [0; 20];
    let mut devices: Vec<String> = vec![];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((_amt, addr)) => {
                
                let ip_addr = addr.ip().to_string();
                devices.push(ip_addr);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                //println!("Timed out");
                break;
            }
            Err(_) => {
                println!("something else");
                break;
            }
        }
    }

    return Ok(devices);
}
