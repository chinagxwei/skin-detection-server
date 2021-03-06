use axum::handler::get;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use axum::response::{Html, IntoResponse};
use axum::{Router, Json};
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};

use crate::{CONFIG, MACHINE_CONTAINER, SUBSCRIPT, MachineID};
use axum::extract::Query;
use crate::mqtt::v3_server::{TopicMessage, ClientID};
use crate::mqtt::message::v3;
use log::{info, debug};
use std::str::FromStr;

#[derive(Serialize)]
pub struct DataResult<T: Serialize> {
    code: u8,
    data: Option<T>,
}

#[derive(Serialize)]
pub struct SimpleDataResult {
    code: u8,
    message: String,
}

impl Default for SimpleDataResult {
    fn default() -> Self {
        SimpleDataResult { code: 1, message: "success".to_string() }
    }
}

impl<T: Serialize> DataResult<T> {
    pub fn new(data: T) -> Self {
        DataResult { code: 1, data: Some(data) }
    }
}

#[derive(Serialize, Deserialize)]
enum MachineMessageEvent {
    LoginEvent = 1,
    SetQrcodeEvent = 2,
}

#[derive(Serialize, Deserialize)]
pub struct MachineMessage {
    id: String,
    event: MachineMessageEvent,
    data: String,
}

impl MachineMessage {
    pub fn qrcode(id: String, url: String) -> Self {
        MachineMessage {
            id,
            event: MachineMessageEvent::SetQrcodeEvent,
            data: url,
        }
    }

    pub fn login(id: String, openid: String) -> Self {
        MachineMessage {
            id,
            event: MachineMessageEvent::LoginEvent,
            data: openid,
        }
    }
}

impl From<MachineQrcode> for MachineMessage {
    fn from(qrcode: MachineQrcode) -> Self {
        MachineMessage {
            id: qrcode.id,
            event: MachineMessageEvent::SetQrcodeEvent,
            data: qrcode.url,
        }
    }
}

impl From<MachineLogin> for MachineMessage {
    fn from(login: MachineLogin) -> Self {
        MachineMessage {
            id: login.id,
            event: MachineMessageEvent::LoginEvent,
            data: login.openid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MachineQrcode {
    id: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MachineLogin {
    id: String,
    openid: String,
}

pub async fn http_server() {
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/machines", get(get_machines))
        .route("/set_machine_qrcode", get(set_machine_qrcode))
        .route("/machine_login", get(machine_login));

    let socket = SocketAddrV4::new(
        Ipv4Addr::from_str(CONFIG.get_http_ip()).unwrap(),
        CONFIG.get_http_port(),
    );
    let addr = SocketAddr::from(socket);
    info!("http listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Skin detection server</h1>")
}

///
/// ??????????????????
///
async fn get_machines() -> impl IntoResponse {
    (StatusCode::OK, MACHINE_CONTAINER.machine_list_json().await)
}

///
/// ?????????????????????
///
async fn set_machine_qrcode(Query(payload): Query<MachineQrcode>) -> impl IntoResponse {
    debug!("{:?}",payload);
    let entity = MachineMessage::from(payload);
    let id = MachineID(entity.id.clone());
    MACHINE_CONTAINER.set_qrcode(&id, entity.data.clone()).await;
    broadcast(entity).await
}

///
/// ????????????????????????????????????????????????openid
///
async fn machine_login(Query(payload): Query<MachineLogin>) -> impl IntoResponse {
    debug!("{:?}",payload);
    let entity = MachineMessage::from(payload);
    broadcast(entity).await
}

async fn broadcast(machine_message: MachineMessage) -> (StatusCode, Json<SimpleDataResult>) {
    let topic = format!("{}-topic", machine_message.id.clone());
    let publish_message = v3::PublishMessage::simple_new_msg(
        topic,
        0,
        serde_json::to_string(&machine_message).unwrap(),
    );
    let topic_msg = TopicMessage::ContentV3(ClientID("idreamspace-server".to_string()), publish_message);
    if let Some(topic) = topic_msg.get_topic() {
        SUBSCRIPT.broadcast(topic, &topic_msg).await;
    }
    (StatusCode::OK, Json(SimpleDataResult::default()))
}
