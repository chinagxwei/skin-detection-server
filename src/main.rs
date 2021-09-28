use skin_detection_server::http::http_server;
use skin_detection_server::mqtt::mqtt_server;


#[tokio::main]
async fn main() {
    tokio::join!(
        mqtt_server(),
        http_server()
    );
}
