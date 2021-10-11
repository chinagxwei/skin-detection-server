use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    http: Option<HttpParam>,
    mqtt: Option<MqttParam>,
    preload: Option<PreloadParam>,
    ping: Option<PingParam>,
}

impl Config {
    pub fn get_http_ip(&self) -> &String {
        &self.http.as_ref().expect("get http ip is error").ip
    }

    pub fn get_http_port(&self) -> u16 {
        self.http.as_ref().expect("get http ip is error").port
    }

    pub fn get_mqtt_ip(&self) -> &String {
        &self.mqtt.as_ref().expect("get mqtt ip is error").ip
    }

    pub fn get_mqtt_port(&self) -> u16 {
        self.mqtt.as_ref().expect("get mqtt ip is error").port
    }

    pub fn get_preload_url(&self) -> &str {
        &self.preload.as_ref().expect("get preload url is error").url
    }

    pub fn get_ping_interval(&self)->u64{
        self.ping.as_ref().expect("get ping interval is error").interval
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpParam {
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MqttParam {
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreloadParam {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingParam {
    pub interval: u64,
}

///
/// 读取文件
///
fn read_file(path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn load_config_file() -> Config {
    let file = read_file("./config/server.toml").expect("read config file error");
    return toml::from_str(String::from(file).trim()).expect("parse config file error");
}
