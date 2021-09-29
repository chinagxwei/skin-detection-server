use skin_detection_server::http::http_server;
use skin_detection_server::mqtt::mqtt_server;


#[tokio::main]
async fn main() {
    log4rs::init_file("./config/log4rs.yml", Default::default()).unwrap();
    tokio::join!(
        mqtt_server(),
        http_server()
    );
}
