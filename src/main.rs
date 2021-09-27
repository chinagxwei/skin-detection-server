use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use skin_detection_server::mqtt::server::{Line,LineMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite};
use skin_detection_server::mqtt::message::MqttMessageKind;

pub struct MqttServer {
    host: String,
    port: u32,
    run_time: Option<Runtime>,
}

impl MqttServer {
    pub fn new<S: Into<String>>(host: S, port: u32) -> MqttServer {
        MqttServer { host: host.into(), port, run_time: None }
    }

    pub fn init(mut self) -> MqttServer {
        if self.run_time.is_none() {
            self.run_time = Option::from(Runtime::new().expect("create tokio is error"));
        }
        self
    }

    pub fn start(&self) {
        if self.run_time.is_none() { return; }
        self.run_time.as_ref().unwrap().block_on(async move {
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
                                    println!("data: {:?}",data);
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
        })
    }
}


fn main() {
    let server = MqttServer::new("127.0.0.1", 22222);
    server.init().start();
}
