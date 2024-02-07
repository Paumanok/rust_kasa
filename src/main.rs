use rust_kasa::{validate_ip, kasa_protocol};
use clap::Parser;
use std::net::TcpStream;
use std::string::String;


#[derive(Parser)]
struct Cli {
    #[arg(short = 't', long = "target_addr")]
    target_addr: String,

    #[arg(short = 'a', long = "action")]
    action: String,
    
}

fn main() -> std::io::Result<()>{
    let args = Cli::parse();
    println!("Hello, world!");
    
    println!("target_addr: {:?}", args.target_addr);

    if validate_ip(&args.target_addr) {
        println!("good ip");
    } else {
        println!("bad ip");
    }
    

    
    let mut stream = TcpStream::connect(args.target_addr +":9999")?;

    match args.action.as_str() {
        "toggle" => kasa_protocol::toggle_relay_by_idx(&mut stream, 0),
        _  => println!("other"),
    };

    let s: kasa_protocol::SysInfo = kasa_protocol::get_sys_info(&mut stream).unwrap();

    
    
    println!("{:?}", s.hwId);
    for child in s.children {
        println!("found child: {:?} Alias: {:?}, state: {:?}", child.id,  child.alias, child.state);   
    }

    let e: Option<Vec<kasa_protocol::Realtime>> = kasa_protocol::get_all_realtime(&mut stream);
    match e {
        None=> println!("get realtime failed"),
        Some(realtime) => println!("{:#?}", realtime),
    }
    Ok(())
}
