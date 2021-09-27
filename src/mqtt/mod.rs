use crate::mqtt::server::{Line, LineMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite};
use tokio::net::TcpListener;
use crate::mqtt::message::MqttMessageKind;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;
pub mod v3_handle;

pub struct MqttServer {
    host: String,
    port: u32,
}

impl MqttServer {
    pub fn new<S: Into<String>>(host: S, port: u32) -> MqttServer {
        MqttServer { host: host.into(), port }
    }

    pub async fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.expect("listener error");

        loop {
            let (mut socket, _) = listener.accept().await.expect("listener accept error");

            tokio::spawn(async move {
                let mut buf = [0; 1024];
                let mut line = Line::new();
                // In a loop, read data from the socket and write the data back.
                'end_loop: loop {
                    let res = tokio::select! {
                            Ok(n) = socket.read(&mut buf) => {
                                if n != 0 {
                                    // println!("length: {}",n);
                                    line.get_sender().send(LineMessage::SocketMessage(buf[0..n].to_vec())).await;
                                }
                                None
                            },
                            kind = line.recv() => kind,
                        };
                    if let Some(kind) = res {
                        match kind {
                            MqttMessageKind::Response(data) => {
                                println!("data: {:?}", data);
                                if let Err(e) = socket.write_all(data.as_slice()).await {
                                    println!("failed to write to socket; err = {:?}", e);
                                }
                            }
                            MqttMessageKind::Exit(data) => {
                                if let Err(e) = socket.write_all(data.as_slice()).await {
                                    println!("failed to write to socket; err = {:?}", e);
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

pub async fn mqtt_server(){
    let server = MqttServer::new("127.0.0.1", 22222);
    server.start().await;
}
