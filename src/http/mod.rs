use axum::handler::get;
use std::net::SocketAddr;
use axum::response::{Html, IntoResponse};
use axum::{Router, Json};
use axum::http::StatusCode;
use serde::{Serialize, Deserialize};

use crate::{MACHINE_CONTAINER, SUBSCRIPT, MachineID};
use axum::extract::Query;
use crate::mqtt::v3_server::{TopicMessage, ClientID};
use crate::mqtt::message::v3;

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
    Login = 1,
    SetQrcode = 2,
}

#[derive(Serialize, Deserialize)]
struct MachineMessage {
    id: String,
    event: MachineMessageEvent,
    data: String,
}

impl From<MachineQrcode> for MachineMessage {
    fn from(qrcode: MachineQrcode) -> Self {
        MachineMessage {
            id: qrcode.id,
            event: MachineMessageEvent::SetQrcode,
            data: qrcode.url,
        }
    }
}

impl From<MachineLogin> for MachineMessage {
    fn from(login: MachineLogin) -> Self {
        MachineMessage {
            id: login.id,
            event: MachineMessageEvent::Login,
            data: login.openid,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MachineQrcode {
    id: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct MachineLogin {
    id: String,
    openid: String,
}

pub async fn http_server() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/machines", get(get_machines))
        .route("/set_machine_qrcode", get(set_machine_qrcode))
        .route("/machine_login", get(machine_login));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    tracing::debug!("listening on {}", addr);
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Skin detection server</h1>")
}

///
/// 返回机器列表
///
async fn get_machines() -> impl IntoResponse {
    (StatusCode::OK, MACHINE_CONTAINER.machine_list_json().await)
}

///
/// 设置机器二维码
///
async fn set_machine_qrcode(Query(payload): Query<MachineQrcode>) -> impl IntoResponse {
    let entity = MachineMessage::from(payload);
    let id = MachineID(entity.id.clone());
    MACHINE_CONTAINER.set_qrcode(&id, entity.data.clone()).await;
    broadcast(entity).await
}

///
/// 告知机器用户已经登录，并返回用户openid
///
async fn machine_login(Query(payload): Query<MachineLogin>) -> impl IntoResponse {
    let entity = MachineMessage::from(payload);
    broadcast(entity).await
}

async fn broadcast(msg: MachineMessage) -> (StatusCode, Json<SimpleDataResult>) {
    let topic = format!("{}-topic", msg.id.clone());
    let msg = v3::PublishMessage::simple_new_msg(
        topic,
        0,
        serde_json::to_string(&msg).unwrap(),
    );
    let topic_msg = TopicMessage::ContentV3(ClientID("idreamspace-server".to_string()), msg);
    if let Some(topic) = topic_msg.get_topic() {
        SUBSCRIPT.broadcast(topic, &topic_msg).await;
    }
    (StatusCode::OK, Json(SimpleDataResult::default()))
}
