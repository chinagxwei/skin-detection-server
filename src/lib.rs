#[macro_use]
extern crate lazy_static;

pub mod mqtt;
pub mod http;

use crate::mqtt::server::Subscript;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use axum::Json;
use crate::http::DataResult;

lazy_static! {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    id: String,
    qrcode_url: String,
    topic: String,
    status: MachineStatus,
}

pub struct MachineManager {
    map: HashMap<MachineID, Machine>,
    interactive_log: Vec<String>,
}

impl MachineManager {
    pub fn new() -> Self {
        MachineManager { map: HashMap::new(), interactive_log: Vec::with_capacity(20) }
    }

    pub fn append(&mut self, id: MachineID, machine: Machine) -> Option<Machine> {
        self.map.insert(id, machine)
    }

    pub fn remove(&mut self, id: &MachineID) -> Option<Machine> {
        self.map.remove(id)
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

    pub async fn append(&self, id: MachineID, machine: Machine) -> Option<Machine> {
        self.container.lock().await.append(id, machine)
    }

    pub async fn remove(&self, id: &MachineID) -> Option<Machine> {
        self.container.lock().await.remove(id)
    }

    pub async fn machine_list_json(&self) -> Json<DataResult<HashMap<MachineID, Machine>>> {
        self.container.lock().await.list_json_result()
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
                topic: "".to_string(),
                status: MachineStatus::Offline,
            },
        );
        manager.append(
            MachineID::new(String::from("2")),
            Machine {
                id: String::from("2"),
                qrcode_url: "".to_string(),
                topic: "".to_string(),
                status: MachineStatus::Offline,
            },
        );
    }
}
