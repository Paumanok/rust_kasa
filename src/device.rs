use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::net::UdpSocket;
use crate::kasa_protocol::{self, decrypt, deserialize, encrypt, get_sys_info, toggle_relay_by_idx};
use crate::models::{KasaChildren, KasaResp, System};
use crate::validate_ip;
use std::net::TcpStream;

pub struct Device {
    pub ip_addr: String,
    pub kasa_info: KasaResp,
    //name?
    //last seen
    //
//
}

impl Device {
    
    pub fn new(ip_addr: String, kasa_info: KasaResp) -> Device {
        Device {
            ip_addr, 
            kasa_info,
        }
    }

    pub fn get_children(&self) -> Option<Vec<KasaChildren>> {
        let stream = TcpStream::connect(self.ip_addr.clone() + ":9999");
        if let Ok(mut strm) = stream {

            let  children = match kasa_protocol::get_children(&mut strm){
                Ok(chldrn) => chldrn,
                Err(_err) => vec![],
            };
            return Some(children);
        }
        //let children = self.kasa_info.system.unwrap().get_sysinfo.unwrap().children.clone();
        println!("failed to get children");
        return None
    }
    
    //make this return the child after the change
    pub fn toggle_relay_by_id(self, idx: usize)  {   
        let  stream = TcpStream::connect(self.ip_addr.clone() + ":9999");
        if let Ok(mut strm) = stream {
             
            let _ = toggle_relay_by_idx(&mut strm, idx);
        }
    }
    
}

pub fn determine_target(t_addr: String) -> Result<Device> {
    if t_addr == "" {
        //try discovery
        if let Ok(kd) = discover() {
            return Ok(kd);
        } else {
            return Err(anyhow!("Discovery failed and no target was provided"));
        }
    } else {
        if validate_ip(&t_addr) {
            println!("good ip");
            let mut stream = TcpStream::connect(t_addr.clone() + ":9999")?;
            return match get_sys_info(&mut stream) {
                Ok(si) => Ok(Device {
                    ip_addr: t_addr,
                    kasa_info: KasaResp {
                        system: Some(System{get_sysinfo: Some(si)}),
                        emeter: None,
                    },
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
pub fn discover() -> Result<Device> {
    let socket = UdpSocket::bind("0.0.0.0:46477")?;
    socket.set_broadcast(true)?;

    let cmd = json!({"system": {"get_sysinfo":0}}).to_string();
    //this is for the newer devices, which I lack
    //let cmd2 = "020000010000000000000000463cb5d3";

    let enc_cmd = encrypt(&cmd, false);
    //println!("{:?}", enc_cmd);
    //let enc_cmd: &[u8] = &enc_cmd;

    socket.send_to(&enc_cmd, "255.255.255.255:9999")?;

    //println!("sent");

    let mut buf = [0; 2048];

    let mut ip_addr : String = String::new();

    if let Ok((n, addr)) = socket.recv_from(&mut buf) {
        //println!("{} bytes response from {:?}", n, addr);
        ip_addr = addr.to_string();
    }

    let mut len: usize = 0;
    while buf[len] != 0 {
        len += 1;
    }


    let mut recv: Vec<u8> = vec![];
    recv.extend_from_slice(&buf[..len]);

    let decrypted = decrypt(&recv);

    let info = deserialize(&decrypted);
    //println!("{}", info.system.unwrap().get_sysinfo.unwrap().sw_ver);

    return Ok(Device::new(ip_addr,info))
}



