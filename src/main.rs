use clap::Parser;
use rust_kasa::models::KasaResp;
use rust_kasa::{device, kasa_protocol, models, validate_ip};
use std::net::TcpStream;
use std::string::String;
use anyhow::{anyhow, Result};

#[derive(Parser)]
struct Cli {
    #[arg(short = 't', long = "target_addr", default_value_t = String::from(""))]
    target_addr: String,

    #[arg(short = 'a', long = "action", default_value_t = String::from(""))]
    action: String,
}

fn determine_target(t_addr: String) -> Result<device::Device> {
    if t_addr == "" {
        //try discovery
        if let Ok(kd) = device::discover() {
            return Ok(kd);
        } else {
            return Err(anyhow!("Discovery failed and no target was provided"));
        }
    } else {
        if validate_ip(&t_addr) {
            println!("good ip");
            let mut stream = TcpStream::connect(t_addr.clone() + ":9999")?;
            return match kasa_protocol::get_sys_info(&mut stream) {
                Ok(si) => Ok(device::Device {
                    ip_addr: t_addr,
                    kasa_info: models::KasaResp {
                        system: Some(models::System{get_sysinfo: Some(si)}),
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

fn main() -> Result<()> {

    let args = Cli::parse();

    let device = determine_target(args.target_addr)?;

    let mut stream = TcpStream::connect(device.ip_addr )?;


    match args.action.as_str() {
        "toggle" => {
            _ = kasa_protocol::toggle_relay_by_idx(&mut stream, 0)
                .unwrap_or_else(|error| panic!("{error:?}"))
        }
        _ => println!("other"),
    };


    let _j: models::SysInfo = kasa_protocol::get_sys_info(&mut stream).unwrap();

    let s: Vec<models::KasaChildren> = kasa_protocol::get_children(&mut stream).unwrap();

    let rt: models::Realtime = kasa_protocol::get_realtime_by_id(&mut stream, &s[0].id).unwrap();

    println!("ma: {:?}", rt.current_ma);

    for child in &s {
        println!(
            "found child: {:?} Alias: {:?}, state: {:?}",
            child.id, child.alias, child.state
        );
    }
    let amp = &s[2];
    let alias_success = kasa_protocol::set_outlet_alias(&mut stream, &amp.id, "amp");

    if let Ok(suc) = alias_success {
        println!("{suc}")
    }

    let s: Vec<models::KasaChildren> = kasa_protocol::get_children(&mut stream).unwrap();

    for child in s {
        println!(
            "found child: {:?} Alias: {:?}, state: {:?}",
            child.id, child.alias, child.state
        );
    }

    let _e: Vec<models::Realtime> = kasa_protocol::get_all_realtime(&mut stream)?;
    Ok(())
}
