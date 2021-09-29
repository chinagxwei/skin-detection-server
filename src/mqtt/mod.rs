use crate::mqtt::v3_server::{Line, LineMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use crate::mqtt::message::MqttMessageKind;
use log::{debug, info};
use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use crate::CONFIG;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod v3_server;
pub mod v3_handle;

pub struct MqttServer {
    addr: SocketAddr,
}

impl MqttServer {
    pub fn new(addr: SocketAddr) -> MqttServer {
        MqttServer { addr }
    }

    pub async fn start(&self) {
        info!("mqtt listening on {}", self.addr);
        let listener = TcpListener::bind(self.addr).await.expect("listener error");

        loop {
            let (mut socket, _) = listener.accept().await.expect("listener accept error");

            tokio::spawn(async move {
                let mut buf = [0; 1024];
                let mut line = Line::new();
                'end_loop: loop {
                    let res = tokio::select! {
                            Ok(n) = socket.read(&mut buf) => {
                                if n != 0 {
                                    line.get_sender().send(LineMessage::SocketMessage(buf[0..n].to_vec())).await;
                                }
                                None
                            },
                            kind = line.recv() => kind,
                        };
                    if let Some(kind) = res {
                        match kind {
                            MqttMessageKind::Response(data) => {
                                debug!("data: {:?}", data);
                                if let Err(e) = socket.write_all(data.as_slice()).await {
                                    debug!("failed to write to socket; err = {:?}", e);
                                }
                            }
                            MqttMessageKind::Exit(data) => {
                                if let Err(e) = socket.write_all(data.as_slice()).await {
                                    debug!("failed to write to socket; err = {:?}", e);
                                }
                                break 'end_loop;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    }
}

pub async fn mqtt_server() {
    let socket = SocketAddrV4::new(
        Ipv4Addr::from_str(CONFIG.get_mqtt_ip()).unwrap(),
        CONFIG.get_mqtt_port()
    );
    let server = MqttServer::new( SocketAddr::from(socket));
    server.start().await;
}
