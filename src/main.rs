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


fn main() -> Result<()> {

    let args = Cli::parse();

    let dev = device::determine_target(args.target_addr)?;

    let mut stream = TcpStream::connect(dev.ip_addr )?;


    match args.action.as_str() {
        "toggle" => {
            _ = kasa_protocol::toggle_relay_by_idx(&mut stream, 0)
                .unwrap_or_else(|error| panic!("{error:?}"))
        }
        "toggle2" => {
            dev.toggle_relay_by_id(0)
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
