use crate::mqtt::v3_server::{Line, LineMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use crate::mqtt::message::MqttMessageKind;
use log::{debug};

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod v3_server;
pub mod v3_handle;

pub struct MqttServer {
    host: [u8; 4],
    port: u32,
}

impl MqttServer {
    pub fn new(host: [u8; 4], port: u32) -> MqttServer {
        MqttServer { host, port }
    }

    pub async fn start(&self) {
        let address = format!("{}.{}.{}.{}:{}", self.host[0], self.host[1], self.host[2], self.host[3], self.port);
        let listener = TcpListener::bind(address).await.expect("listener error");

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
    let server = MqttServer::new([127, 0, 0, 1], 22222);
    server.start().await;
}
