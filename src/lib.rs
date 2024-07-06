pub mod kasa_protocol;
pub mod models;

pub fn validate_ip(ip: &String) -> bool {
    let ip: Vec<&str> = ip.split(".").collect();

    if ip.len() != 4 {
        return false;
    }
    for octet in ip {
        let _val: u8 = match octet.parse() {
            Ok(value) => value,
            Err(_error) => return false,
        };
    }
    return true;
}
