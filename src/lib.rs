#[macro_use]
extern crate lazy_static;

pub mod mqtt;
pub mod http;
mod config;

use crate::mqtt::v3_server::Subscript;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use axum::Json;
use crate::http::DataResult;
use crate::config::{Config, load_config_file};

lazy_static! {
    static ref CONFIG: Config = load_config_file();
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
    pub static ref MACHINE_CONTAINER: MachineContainer = MachineContainer::new();
}

#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct MachineID(String);

impl MachineID {
    pub fn new(field0: String) -> Self {
        MachineID(field0)
    }
}

impl PartialEq for MachineID {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self.0, &other.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineStatus {
    Offline = 0,
    Online,
}

impl MachineStatus {
    pub fn is_online(&self) -> bool {
        matches!(self, MachineStatus::Online)
    }

    pub fn is_offline(&self) -> bool {
        matches!(self, MachineStatus::Offline)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    id: String,
    qrcode_url: String,
    status: MachineStatus,
}

impl Machine {
    pub fn online(&mut self) {
        self.status = MachineStatus::Online;
    }

    pub fn offline(&mut self) {
        self.status = MachineStatus::Offline;
    }

    pub fn set_qrcode_url(&mut self, qrcode_url: String) {
        self.qrcode_url = qrcode_url;
    }
}

pub struct MachineManager {
    map: HashMap<MachineID, Machine>,
}

impl MachineManager {
    pub fn new() -> Self {
        MachineManager { map: HashMap::new() }
    }

    pub fn init_map(&mut self, map: HashMap<MachineID, Machine>) {
        self.map = map;
    }

    pub fn set_qrcode(&mut self, id: &MachineID, qrcode_url: String) {
        if self.map.contains_key(id) {
            let item = self.map.get_mut(&id).expect("append machine error");
            item.set_qrcode_url(qrcode_url);
        }
    }

    pub fn append(&mut self, id: MachineID, machine: Machine) {
        if self.map.contains_key(&id) {
            let item = self.map.get_mut(&id).expect("append machine error");
            item.online();
        } else {
            self.map.insert(id, machine);
        }
    }

    pub fn remove(&mut self, id: &MachineID) {
        if self.map.contains_key(&id) {
            let item = self.map.get_mut(&id).expect("remove machine error");
            item.offline();
        }
    }

    pub fn list_json_result(&self) -> Json<DataResult<HashMap<MachineID, Machine>>> {
        Json(DataResult::new(self.map.clone()))
    }
}

pub struct MachineContainer {
    container: Arc<Mutex<MachineManager>>,
}

impl MachineContainer {
    pub fn new() -> Self {
        MachineContainer { container: Arc::new(Mutex::new(MachineManager::new())) }
    }

    pub async fn append(&self, id: MachineID, machine: Machine) {
        self.container.lock().await.append(id, machine);
    }

    pub async fn remove(&self, id: &MachineID) {
        self.container.lock().await.remove(id);
    }

    pub async fn machine_list_json(&self) -> Json<DataResult<HashMap<MachineID, Machine>>> {
        self.container.lock().await.list_json_result()
    }

    pub async fn init_machines(&self, machines: HashMap<MachineID, Machine>) {
        self.container.lock().await.init_map(machines);
    }

    pub async fn set_qrcode(&self, id: &MachineID, qrcode_url: String) {
        self.container.lock().await.set_qrcode(id, qrcode_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut manager = MachineManager::new();
        manager.append(
            MachineID::new(String::from("1")),
            Machine {
                id: String::from("1"),
                qrcode_url: "".to_string(),
                status: MachineStatus::Offline,
            },
        );
        manager.append(
            MachineID::new(String::from("2")),
            Machine {
                id: String::from("2"),
                qrcode_url: "".to_string(),
                status: MachineStatus::Offline,
            },
        );
    }
}

