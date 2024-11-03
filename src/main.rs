use anyhow::{anyhow, Result};
use clap::Parser;
use rust_kasa::models::KasaResp;
use rust_kasa::{device, kasa_protocol, models, validate_ip};
use std::any::Any;
use std::io::stdout;
use std::net::TcpStream;
use std::string::String;
mod app;

use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Parser)]
struct Cli {
    #[arg(short = 't', long = "target_addr", default_value_t = String::from(""))]
    target_addr: String,

    #[arg(short = 'n', long = "name", default_value_t = String::from(""))]
    target_name: String,

    #[arg(short = 'a', long = "action", default_value_t = String::from(""))]
    action: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.target_name != "" {
        println!("does this work");
    }

    //let dev = device::determine_target(args.target_addr)?;

    //let mut stream = TcpStream::connect(dev.ip_addr.clone() )?;

    //match args.action.as_str() {
    //    "toggle" => {
    //        _ = kasa_protocol::toggle_relay_by_idx(&mut stream, 0)
    //            .unwrap_or_else(|error| panic!("{error:?}"))
    //    }
    //    "toggle2" => {
    //        dev.toggle_relay_by_id(0)
    //    }
    //    _ => println!("other"),
    //};

    //let _j: models::SysInfo = kasa_protocol::get_sys_info(&mut stream).unwrap();

    //let s: Vec<models::KasaChildren> = kasa_protocol::get_children(&mut stream).unwrap();

    //let rt: models::Realtime = kasa_protocol::get_realtime_by_id(&mut stream, &s[0].id).unwrap();

    //println!("ma: {:?}", rt.current_ma);

    //for child in &s {
    //    println!(
    //        "found child: {:?} Alias: {:?}, state: {:?}",
    //        child.id, child.alias, child.state
    //    );
    //}
    ////let amp = &s[2];
    ////let alias_success = kasa_protocol::set_outlet_alias(&mut stream, &amp.id, "amp");

    ////if let Ok(suc) = alias_success {
    ////    println!("{suc}")
    ////}

    //let s: Vec<models::KasaChildren> = kasa_protocol::get_children(&mut stream).unwrap();

    //for child in s {
    //    println!(
    //        "found child: {:?} Alias: {:?}, state: {:?}",
    //        child.id, child.alias, child.state
    //    );
    //}

    //let _e: Vec<models::Realtime> = kasa_protocol::get_all_realtime(&mut stream)?;

    //if let Ok(dev) = device::discover() {
    //    println!("ip: {:?}", dev.ip_addr);
    //}

    //if let Ok(devices) = device::discover_multiple() {
    //    for dev in devices {
    //        println!("ip: {:?}", dev.ip_addr);
    //        print!("info {:}\n", dev.sysinfo().unwrap());
    //    }
    //}
    let terminal = ratatui::init();
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alternate screen");
    let app_result = App::default().run(terminal);
    execute!(stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");
    ratatui::restore();
    app_result
    //Ok(())
}
