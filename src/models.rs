use serde::{Deserialize, Serialize};

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
    pub latitude_i: u32,
    pub led_off: u8,
    pub longitude_i: u32,
    pub mac: String,
    pub mic_type: String,
    pub model: String,
    pub oemId: String,
    pub rssi: i32,
    pub status: String,
    pub sw_ver: String,
    pub updating: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Realtime {
    pub current_ma: u32,
    pub err_code: u32,
    pub power_mw: u32,
    pub slot_id: u32,
    pub total_wh: u32,
    pub voltage_mv: u32,
}

#[derive(Serialize, Deserialize)]
pub struct System {
    pub get_sysinfo: Option<SysInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct Emeter {
    pub get_realtime: Option<Realtime>,
}

#[derive(Serialize, Deserialize)]
pub struct KasaResp {
    pub system: Option<System>,
    pub emeter: Option<Emeter>,
}
