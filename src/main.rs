use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use skin_detection_server::mqtt::server::{Line,LineMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite};
use skin_detection_server::mqtt::message::MqttMessageKind;

use skin_detection_server::http::http_server;
use skin_detection_server::mqtt::mqtt_server;


#[tokio::main]
async fn main() {
    let (first, second) = tokio::join!(
        mqtt_server(),
        http_server()
    );
}
